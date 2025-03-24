// core/src/audio/decoder.rs

use std::fs::File;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use symphonia::core::{
    audio::{SampleBuffer, SignalSpec},
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymphError,
    formats::{FormatOptions, SeekMode},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};

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

    let file = File::open(path).map_err(|e| anyhow!("Error opening file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

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
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| anyhow!("Error probing media format: {}", e))?;

    let mut format = probed.format;
    // Clone the track info to release the immutable borrow of format
    let track_info = {
        let t = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow!("No supported audio track found"))?;
        t.clone()
    };

    let track_codec_params = track_info.codec_params.clone();
    let track_id = track_info.id;
    let channels = track_codec_params
        .channels
        .unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT);
    let channel_count = channels.count();
    let file_sample_rate = track_codec_params.sample_rate.unwrap_or(44100);

    info!("Found audio track: {:?}, {} ch, {} Hz",
          track_codec_params.codec, channel_count, file_sample_rate);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track_codec_params, &decoder_opts)
        .map_err(|e| anyhow!("Error creating audio decoder: {}", e))?;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("No output audio device available"))?;

    let mut config_range = device
        .supported_output_configs()
        .map_err(|e| anyhow!("Failed to get device configs: {}", e))?
        .filter(|c| c.channels() >= channel_count as u16)
        .collect::<Vec<_>>();

    if config_range.is_empty() {
        return Err(anyhow!("No suitable output config found for device"));
    }

    config_range.sort_by_key(|c| c.min_sample_rate().0);
    let desired_sample_rates = [file_sample_rate, 48000, 44100, 96000, 192000];
    let mut selected_config = None;

    for &rate in &desired_sample_rates {
        for c in &config_range {
            if rate >= c.min_sample_rate().0 && rate <= c.max_sample_rate().0 {
                selected_config = Some(c.with_sample_rate(cpal::SampleRate(rate)));
                break;
            }
        }
        if selected_config.is_some() {
            break;
        }
    }

    let device_config = selected_config.unwrap_or_else(|| {
        config_range[0].clone().with_sample_rate(config_range[0].min_sample_rate())
    });
    let config = device_config.config();
    let output_sample_rate = config.sample_rate.0;

    let buffer_size_frames = (output_sample_rate as usize * channel_count as usize) / 10;
    let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(buffer_size_frames * 4)));
    let ring_buffer_stream = Arc::clone(&ring_buffer);

    let needs_data = Arc::new(AtomicBool::new(true));
    let needs_data_stream = Arc::clone(&needs_data);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _info| {
            let needed = data.len();
            let mut samples = vec![0.0; needed];
            if let Ok(mut rb) = ring_buffer_stream.lock() {
                let read_count = rb.read(&mut samples);
                if read_count < needed {
                    for s in &mut data[read_count..] {
                        *s = 0.0;
                    }
                }
                data[..read_count].copy_from_slice(&samples[..read_count]);

                if rb.available() < buffer_size_frames / 2 {
                    needs_data_stream.store(true, Ordering::Release);
                }
            } else {
                for s in data.iter_mut() {
                    *s = 0.0;
                }
            }
        },
        |err| {
            error!("Audio output error: {}", err);
        },
        None,
    )?;

    stream.play().map_err(|e| anyhow!("Failed to start audio: {}", e))?;

    let total_samples = if let Some(n_frames) = track_codec_params.n_frames {
        n_frames.saturating_mul(channel_count as u64)
    } else {
        (file_sample_rate as u64) * channel_count as u64 * 300
    };

    {
        let mut pos = playback_position.lock().unwrap();
        pos.set_total_samples(total_samples);
        pos.set_channel_count(channel_count); // Set the channel count correctly
        pos.sample_rate = file_sample_rate;
    }

    let track_duration_seconds =
        (total_samples as f64) / (file_sample_rate.max(1) as f64 * channel_count as f64);
    let duration = Duration::from_secs_f64(track_duration_seconds);

    if let Ok(mut st) = state_arc.lock() {
        st.duration = Some(duration);
    }

    let spec = SignalSpec::new(file_sample_rate, channels);
    let buffer_frames: usize = 2048;
    let mut sample_buf = SampleBuffer::<f32>::new(buffer_frames as u64, spec);

    let mut current_frames: u64 = 0;

    info!("Starting decode loop for track: {}", path);
    let mut is_eof = false;
    let mut last_debug_log = Instant::now();

    while !is_eof {
        if stop_flag.load(Ordering::SeqCst) {
            info!("Stop signal received; breaking decode loop");
            break;
        }
        if pause_flag.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        let mut do_seek = false;
        let mut target_fraction = 0.0;
        {
            if let Ok(pos_lock) = playback_position.lock() {
                if let Some(req_flag) = &pos_lock.seek_requested {
                    if req_flag.swap(false, Ordering::SeqCst) {
                        do_seek = true;
                        if let Some(tgt) = &pos_lock.seek_target {
                            if let Ok(tgt_val) = tgt.lock() {
                                target_fraction = *tgt_val;
                            }
                        }
                    }
                }
            }
        }

        if do_seek {
            info!("Seek requested -> fraction = {:.4} ({:.2}%)", 
                 target_fraction, target_fraction * 100.0);
            
            // Clear the ring buffer first to prevent old audio from playing
            if let Ok(mut rb) = ring_buffer.lock() {
                *rb = AudioRingBuffer::new(rb.capacity());
            }
            
            // For better debugging: calculate frames vs samples
            let frame_count = total_samples / channel_count as u64;
            
            // For very precise seek calculation
            let target_time_seconds = target_fraction * track_duration_seconds as f32;
            info!("Target time in seconds: {:.4} of total {:.4}s", 
                 target_time_seconds, track_duration_seconds);
            
            // Convert to Symphonia Time format
            let whole_secs = target_time_seconds as u64;
            let frac = target_time_seconds - whole_secs as f32;
            let seek_time = Time { seconds: whole_secs, frac: frac as f64 };
            
            info!("Seeking to time: {}s + {:.6}s", seek_time.seconds, seek_time.frac);
            
            // Perform the actual seek with more debugging
            match format.seek(
                SeekMode::Accurate,  // Use accurate mode for better precision
                symphonia::core::formats::SeekTo::Time {
                    track_id: Some(track_id),
                    time: seek_time,
                },
            ) {
                Ok(seeked_to) => {
                    info!("Format seek succeeded to actual position: {} samples",
                         seeked_to.actual_ts);
                    
                    // More detailed logging about the actual frame calculation 
                    info!("Seek details: actual_ts={}, total_samples={}, frame_count={}, channel_count={}",
                         seeked_to.actual_ts, total_samples, frame_count, channel_count);
                    
                    // Reset the decoder to ensure clean state
                    decoder.reset();
                    
                    // Calculate the actual frame position
                    let new_frames;
                    if let Some(tb) = track_info.codec_params.time_base {
                        let time = tb.calc_time(seeked_to.actual_ts);
                        // Calculate the frame position
                        let sample_pos = ((time.seconds as f64 + time.frac) * 
                                         file_sample_rate as f64) as u64;
                                         
                        // Convert to frame position (sample position * channel count)
                        new_frames = sample_pos * channel_count as u64;
                        
                        info!("Time-based calculation: time={}s+{:.6}s, sample_pos={}, new_frames={}",
                             time.seconds, time.frac, sample_pos, new_frames);
                    } else {
                        // Fallback calculation directly from target fraction
                        new_frames = (target_fraction * total_samples as f32).round() as u64;
                        info!("Using fallback frame calculation: {}", new_frames);
                    }
                    
                    // Update frames counter 
                    current_frames = new_frames;
                    
                    // Update the playback position atomically with better synchronization
                    if let Ok(pos) = playback_position.lock() {
                        // IMPORTANT: Store the frame index, not the sample index
                        let frame_pos = (current_frames / channel_count as u64) as usize;
                        pos.set_current_frame(frame_pos);
                        
                        // Calculate actual progress for logging
                        let progress = if frame_count > 0 {
                            frame_pos as f64 / frame_count as f64
                        } else {
                            0.0
                        };
                        
                        info!("Updated frame position to {} of {} ({:.4}% of total)",
                             frame_pos, frame_count, progress * 100.0);
                    }
                    
                    // Ensure we fetch new data immediately
                    needs_data.store(true, Ordering::Release);
                }
                Err(e) => {
                    warn!("Seeking failed: {}", e);
                    // Even if format seeking fails, try to update the position
                    let new_frame_index = (target_fraction * frame_count as f32).round() as usize;
                    if let Ok(pos) = playback_position.lock() {
                        pos.set_current_frame(new_frame_index);
                        info!("Updated frame position to {} (fallback after seek error)", 
                             new_frame_index);
                    }
                }
            }
            continue;
        }

        if !needs_data.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(1));
            continue;
        }
        needs_data.store(false, Ordering::Release);

        match format.next_packet() {
            Ok(packet) => {
                if packet.track_id() != track_id {
                    continue;
                }

                match decoder.decode(&packet) {
                    Ok(decoded) => {
                        let frames_decoded = decoded.frames();
                        let needed_capacity = frames_decoded * channel_count;
                        if sample_buf.capacity() < needed_capacity as usize {
                            sample_buf = SampleBuffer::<f32>::new(needed_capacity as u64, spec);
                        }
                        sample_buf.copy_interleaved_ref(decoded);

                        let volume = {
                            if let Ok(v) = volume_arc.lock() {
                                *v
                            } else {
                                1.0
                            }
                        };
                        let raw_samples = sample_buf.samples();
                        let mut volume_applied = Vec::with_capacity(raw_samples.len());
                        for &smp in raw_samples {
                            volume_applied.push(smp * volume);
                        }

                        let final_samples = if file_sample_rate != output_sample_rate {
                            resample(
                                &volume_applied,
                                file_sample_rate,
                                output_sample_rate,
                                channel_count,
                            )
                        } else {
                            volume_applied
                        };

                        if let Ok(mut rb) = ring_buffer.lock() {
                            let _ = rb.write(&final_samples);
                        }

                        current_frames = current_frames.saturating_add(frames_decoded as u64);

                        if let Ok(pos) = playback_position.lock() {
                            // Update with per-frame count, not per-sample count
                            pos.update_current_sample(raw_samples.len());
                        }
                    }
                    Err(e) => {
                        warn!("Decode error: {}", e);
                    }
                }
            }
            Err(err) => {
                if let SymphError::IoError(io_err) = &err {
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                        info!("End of file reached (EOF)");
                        is_eof = true;
                        
                        // Track was played completely, increment the play count
                        // We need to pass this information back to update the track
                        if let Ok(mut st) = state_arc.lock() {
                            st.track_completed = true;  // Set the flag to indicate track completion
                        }
                        
                        continue;
                    }
                }
                warn!("Error reading packet: {}", err);
            }
        }

        if last_debug_log.elapsed() >= Duration::from_millis(200) {
            let cur_seconds = current_frames as f64 / (file_sample_rate.max(1) as f64 * channel_count as f64);
            debug!(
                "Current decoded frames: {} => time ~ {:.3}s / track len {:.3}s",
                current_frames,
                cur_seconds,
                track_duration_seconds
            );
            last_debug_log = Instant::now();
        }
    }

    info!("Decode loop exited. Draining buffer...");
    let drain_start = Instant::now();
    while drain_start.elapsed() < Duration::from_secs(1) {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        let empty = match ring_buffer.lock() {
            Ok(rb) => rb.available() == 0,
            Err(_) => true,
        };
        if empty {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    info!("Playback finished or stopped.");
    Ok(())
}