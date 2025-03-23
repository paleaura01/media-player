// core/src/audio/decoder.rs
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL}; // <--- We keep only what's used
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::audio::buffer::AudioRingBuffer;
use crate::audio::position::PlaybackPosition;
use crate::audio::resampler::resample;
use crate::PlayerState;

pub fn play_audio_file(
    path: &str,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    state_arc: Arc<Mutex<PlayerState>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume_arc: Arc<Mutex<f32>>,
) -> Result<()> {
    info!("Opening file: {}", path);

    if !Path::new(path).exists() {
        error!("File does not exist: {}", path);
        return Err(anyhow!("File does not exist: {}", path));
    }

    let file = match File::open(path) {
        Ok(f) => Box::new(f),
        Err(e) => {
            error!("Failed to open file {}: {}", path, e);
            return Err(anyhow!("Error opening file: {}", e));
        }
    };

    let mss = MediaSourceStream::new(file, Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension() {
        if let Some(ext_str) = ext.to_str() {
            hint.with_extension(ext_str);
            debug!("Detected file extension: {}", ext_str);
        }
    }

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    info!("Probing media...");
    let probed = match symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
    {
        Ok(p) => p,
        Err(e) => {
            error!("Error probing media: {}", e);
            return Err(anyhow!("Error probing media format: {}", e));
        }
    };

    let mut format = probed.format;
    let format_name = if let Some(md) = format.metadata().current() {
        md.tags()
            .iter()
            .find(|tag| tag.key.eq_ignore_ascii_case("title"))
            .map(|tag| tag.value.to_string())
            .unwrap_or_else(|| {
                Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            })
    } else {
        Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    };

    info!("Playing: {}", format_name);

    // Choose the first supported (non-null) track
    let track = match format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
    {
        Some(t) => t,
        None => {
            error!("No supported audio track found in file");
            return Err(anyhow!("No supported audio track found"));
        }
    };

    info!("Found audio track: {}", track.codec_params.codec);

    // Build a decoder
    let mut decoder = match symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
    {
        Ok(d) => d,
        Err(e) => {
            error!("Error creating decoder: {}", e);
            return Err(anyhow!("Error creating audio decoder: {}", e));
        }
    };

    let track_id = track.id;
    let channels = track.codec_params.channels
        .unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT);
    let channel_count = channels.count();
    let file_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    info!("Audio track parameters: {} Hz, {} channels",
          file_sample_rate, channel_count);

    let total_samples = if let Some(n_frames) = track.codec_params.n_frames {
        match n_frames.checked_mul(channel_count as u64) {
            Some(samples) => samples,
            None => {
                warn!("Sample count overflow, using fallback");
                (file_sample_rate as u64) * (channel_count as u64) * 300
            }
        }
    } else {
        (file_sample_rate as u64) * (channel_count as u64) * 300
    };

    if let Ok(mut position) = playback_position.lock() {
        position.set_total_samples(total_samples);
        position.sample_rate = file_sample_rate;
    }

    let duration = {
        let sr = file_sample_rate.max(1) as f64;
        let cc = channel_count.max(1) as f64;
        Duration::from_secs_f64(total_samples as f64 / (sr * cc))
    };

    info!("Track duration: {:?}", duration);

    if let Ok(mut state) = state_arc.lock() {
        state.duration = Some(duration);
    }

    let host = cpal::default_host();
    info!("Using audio host: {}", host.id().name());

    let device = match host.default_output_device() {
        Some(d) => d,
        None => {
            error!("No output audio device available");
            return Err(anyhow!("No output audio device available"));
        }
    };

    info!("Using output device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));

    let config_range = match device.supported_output_configs() {
        Ok(configs) => configs
            .filter(|c| c.channels() >= channel_count as u16)
            .collect::<Vec<_>>(),
        Err(e) => {
            error!("Failed to get device configs: {}", e);
            return Err(anyhow!("Failed to get audio device configurations"));
        }
    };

    if config_range.is_empty() {
        return Err(anyhow!("No suitable output config found for device"));
    }

    let desired_sample_rates = [44100, 48000, 96000, 192000];
    let mut selected_config = None;

    // Attempt exact matching with file sample rate
    for &rate in &desired_sample_rates {
        if rate == file_sample_rate {
            for config in &config_range {
                if rate >= config.min_sample_rate().0 && rate <= config.max_sample_rate().0 {
                    selected_config = Some(config.with_sample_rate(cpal::SampleRate(rate)));
                    info!("Selected output sample rate: {} Hz (exact match)", rate);
                    break;
                }
            }
        }
        if selected_config.is_some() {
            break;
        }
    }

    // If no exact match, prefer 44.1k
    if selected_config.is_none() {
        for config in &config_range {
            if 44100 >= config.min_sample_rate().0
                && 44100 <= config.max_sample_rate().0
            {
                selected_config = Some(config.with_sample_rate(cpal::SampleRate(44100)));
                info!("Selected 44100 Hz (preferred for music)");
                break;
            }
        }
    }

    // If still none, fallback
    let device_config = selected_config.unwrap_or_else(|| {
        let config = &config_range[0];
        let sample_rate = if file_sample_rate <= config.min_sample_rate().0 {
            config.min_sample_rate().0
        } else if file_sample_rate >= config.max_sample_rate().0 {
            config.max_sample_rate().0
        } else {
            file_sample_rate
        };
        info!("Using fallback sample rate: {}", sample_rate);
        config.with_sample_rate(cpal::SampleRate(sample_rate))
    });

    let output_sample_rate = device_config.sample_rate().0;
    let config = device_config.config();

    info!("Output device config: {:?}", config);

    // Create ring buffer
    let buffer_size = ((output_sample_rate as usize * channel_count as usize) / 10).max(1024);
    let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(buffer_size * 4)));
    let ring_buffer_stream = Arc::clone(&ring_buffer);

    let needs_data = Arc::new(AtomicBool::new(true));
    let needs_data_stream = Arc::clone(&needs_data);

    // Start building stream
    info!("Building audio output stream...");
    let stream = match device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let get_samples = || -> (Vec<f32>, usize) {
                // Acquire ring buffer lock
                if let Ok(mut buffer) = ring_buffer_stream.lock() {
                    // If low on samples, request more
                    if buffer.available() < buffer_size / 2 {
                        needs_data_stream.store(true, Ordering::Release);
                    }
                    let mut samples = vec![0.0; data.len()];
                    let count = buffer.read(&mut samples);
                    (samples, count)
                } else {
                    (Vec::new(), 0)
                }
            };

            let result = std::panic::catch_unwind(|| get_samples());

            match result {
                Ok((samples, count)) => {
                    if count > 0
                        && count <= data.len()
                        && count <= samples.len()
                    {
                        data[..count].copy_from_slice(&samples[..count]);
                    }
                    // Fill remainder with silence
                    if count < data.len() {
                        for sample in &mut data[count..] {
                            *sample = 0.0;
                        }
                    }
                },
                Err(_) => {
                    // On panic, fill with silence
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                }
            }
        },
        |err| {
            error!("Audio output error: {}", err);
        },
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to build audio stream: {}", e);
            return Err(anyhow!("Failed to build audio stream: {}", e));
        }
    };

    info!("Starting audio playback stream");
    if let Err(e) = stream.play() {
        error!("Failed to start audio stream: {}", e);
        return Err(anyhow!("Failed to start audio playback: {}", e));
    }

    info!("Audio stream started");
    let spec = SignalSpec::new(file_sample_rate, channels);
    let buffer_frames: usize = 2048;
    let mut sample_buf = SampleBuffer::<f32>::new(buffer_frames as u64, spec);

    info!("Starting decode loop");
    let mut is_eof = false;
    let mut total_samples_processed = 0;
    let mut last_progress_update = Instant::now();

    // Main decode loop
    while !is_eof {
        // Stop check
        if stop_flag.load(Ordering::SeqCst) {
            info!("Stop signal received; exiting decode loop");
            break;
        }

        // Pause check
        if pause_flag.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        // If we have enough data, wait
        if !needs_data.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(1));
            continue;
        }
        needs_data.store(false, Ordering::Release);

        // Update progress ~ every 50ms
        if last_progress_update.elapsed() >= Duration::from_millis(50) {
            if let Ok(position) = playback_position.lock() {
                position.update_current_sample(total_samples_processed);
            }
            total_samples_processed = 0;
            last_progress_update = Instant::now();
        }

        // **Here** we specify an explicit error type for the next_packet() result
        let packet_result: Result<symphonia::core::formats::Packet, symphonia::core::errors::Error>
            = match format.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != track_id {
                        continue;
                    }
                    Ok(packet)
                },
                Err(e) => {
                    if let symphonia::core::errors::Error::IoError(ref io_err) = e {
                        if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                            is_eof = true;
                            info!("End of file reached");
                            continue;
                        }
                    }
                    warn!("Error reading packet: {}", e);
                    Err(e)
                }
            };

        if let Ok(packet) = packet_result {
            // Decode
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let decoded_frames = decoded.frames();
                    if sample_buf.capacity() < (decoded_frames * channel_count) as usize {
                        let needed_capacity = decoded_frames * channel_count;
                        let new_capacity = needed_capacity.max(buffer_frames);
                        sample_buf = SampleBuffer::<f32>::new(new_capacity as u64, spec);
                    }

                    sample_buf.copy_interleaved_ref(decoded);
                    let samples = sample_buf.samples();

                    // Volume
                    let volume = match volume_arc.lock() {
                        Ok(v) => {
                            let vol = *v;
                            if total_samples_processed == 0 {
                                debug!("Current volume level: {:.2}", vol);
                            }
                            vol
                        },
                        Err(e) => {
                            error!("Failed to get volume lock: {}, using default", e);
                            0.8
                        }
                    };

                    // Create a volume-adjusted copy
                    let mut volume_adjusted = Vec::with_capacity(samples.len());
                    for &sample in samples {
                        volume_adjusted.push(sample * volume);
                    }

                    // Update progress count
                    let sample_count = volume_adjusted.len();
                    if sample_count > 0 {
                        total_samples_processed = total_samples_processed.saturating_add(sample_count);
                    }

                    // Resample if needed
                    let output_samples = if file_sample_rate != output_sample_rate
                        && !volume_adjusted.is_empty()
                    {
                        resample(
                            &volume_adjusted,
                            file_sample_rate,
                            output_sample_rate,
                            channel_count,
                        )
                    } else {
                        volume_adjusted
                    };

                    // Write to ring buffer
                    if !output_samples.is_empty() {
                        if let Ok(mut buffer) = ring_buffer.lock() {
                            let _ = buffer.write(&output_samples);
                        }
                    }
                },
                Err(e) => {
                    warn!("Error decoding packet: {}", e);
                }
            }
        } else {
            // If we hit an error, or the packet was Err => either log or break
            // We already handled logging above, so set EOF:
            is_eof = true;
        }
    }

    info!("Decoding complete");

    // Drain leftover samples
    let wait_start = Instant::now();
    while wait_start.elapsed() < Duration::from_secs(1) {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        let buffer_empty = match ring_buffer.lock() {
            Ok(buffer) => buffer.available() == 0,
            Err(_) => true,
        };
        if buffer_empty {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    info!("Playback finished");
    Ok(())
}
