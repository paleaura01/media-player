// src/audio.rs
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use std::time::Duration;

use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use symphonia::core::audio::{SampleBuffer, SignalSpec, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub fn init() -> Result<()> {
    log::info!("Audio subsystem initialized");
    Ok(())
}

/// Decode the given MP3 file and play it through the default audio output without resampling.
pub fn play_audio_file(path: &str, pause_flag: Arc<AtomicBool>, stop_flag: Arc<AtomicBool>) -> Result<()> {
    log::info!("Opening file: {}", path);
    let file = Box::new(File::open(path)?);
    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension() {
        if let Some(ext_str) = ext.to_str() {
            hint.with_extension(ext_str);
            log::debug!("Detected file extension: {}", ext_str);
        }
    }

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    log::info!("Probing media...");
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| anyhow!("Error probing media: {}", e))?;
    let mut format = probed.format;

    let format_name = if let Some(md) = format.metadata().current() {
        md.tags().iter()
            .find(|tag| tag.key.eq_ignore_ascii_case("title"))
            .map(|tag| tag.value.to_string())
            .unwrap_or_else(|| Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string())
    } else {
        Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string()
    };
    log::info!("Playing: {}", format_name);

    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow!("No supported audio track found"))?;
    log::info!("Found audio track: {}", track.codec_params.codec);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|e| anyhow!("Error creating decoder: {}", e))?;

    let track_id = track.id;
    let channels = track.codec_params.channels.unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT);
    let channel_count = channels.count();
    let file_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    log::info!("Audio track parameters: {} Hz, {} channels", file_sample_rate, channel_count);

    // Setup CPAL audio output. We try to match the file's sample rate.
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| anyhow!("No output audio device available"))?;
    let supported_configs = device.supported_output_configs()?;
    // Try to find a config matching the file sample rate
    let matching_config = supported_configs
        .filter(|cfg| cfg.channels() >= channel_count as u16)
        .find(|cfg| {
            let min = cfg.min_sample_rate().0;
            let max = cfg.max_sample_rate().0;
            file_sample_rate >= min && file_sample_rate <= max
        });
    let stream_config = if let Some(cfg) = matching_config {
        cfg.with_sample_rate(cpal::SampleRate(file_sample_rate)).config()
    } else {
        // Fallback to the default config
        device.default_output_config()?.config()
    };
    log::info!("Output device: {} with config: {:?}", 
        device.name().unwrap_or_else(|_| "Unknown".to_string()), stream_config);

    // We expect the file sample rate to match the output.
    if file_sample_rate != stream_config.sample_rate.0 {
        log::warn!("File sample rate {} Hz does not match output device rate {} Hz. Playback speed may be incorrect.",
            file_sample_rate, stream_config.sample_rate.0);
    } else {
        log::info!("No resampling needed; sample rates match.");
    }

    // Create a channel to pass decoded PCM data from the decoding thread to the audio callback.
    let (sample_tx, sample_rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(10);

    // Create an Arc<Mutex> to store pending samples in the callback.
    let pending_samples = Arc::new(Mutex::new(Vec::<f32>::new()));

    // Build and start the CPAL output stream with a callback that manages leftover samples.
    let err_flag = Arc::new(AtomicBool::new(false));
    let err_flag_clone = Arc::clone(&err_flag);
    let pending_samples_cb = Arc::clone(&pending_samples);
    let stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Accumulate any new samples from the channel.
            {
                let mut pending = pending_samples_cb.lock().unwrap();
                while let Ok(buffer) = sample_rx.try_recv() {
                    pending.extend(buffer);
                }
            }
            // Fill the output buffer from the pending samples.
            let mut pending = pending_samples_cb.lock().unwrap();
            let available = pending.len();
            if available >= data.len() {
                data.copy_from_slice(&pending[..data.len()]);
                pending.drain(..data.len());
            } else {
                if available > 0 {
                    data[..available].copy_from_slice(&pending);
                    pending.clear();
                }
                for sample in &mut data[available..] {
                    *sample = 0.0;
                }
            }
        },
        move |err| {
            log::error!("Audio output error: {}", err);
            err_flag_clone.store(true, Ordering::SeqCst);
        },
        None,
    )?;
    stream.play()?;
    log::info!("Audio stream started");

    // Create a sample buffer for decoded audio.
    let spec = SignalSpec::new(file_sample_rate, channels);
    let mut sample_buf = SampleBuffer::<f32>::new(8192, spec);

    log::info!("Entering decode loop");
    // Decoding loop: decode packets and send PCM data to the audio callback.
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            log::info!("Stop signal received; exiting decode loop");
            break;
        }
        if pause_flag.load(Ordering::SeqCst) {
            log::debug!("Playback paused; sleeping for 50 ms");
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }

        let packet = match format.next_packet() {
            Ok(packet) => {
                log::debug!("Packet received (track_id = {})", packet.track_id());
                packet
            },
            Err(symphonia::core::errors::Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                log::info!("End of stream reached");
                break;
            },
            Err(e) => {
                log::warn!("Error reading packet: {}", e);
                continue;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Use copy_interleaved_ref from the Signal trait.
                sample_buf.copy_interleaved_ref(decoded);
                let samples = sample_buf.samples();
                log::debug!("Decoded {} samples", samples.len());
                // Send the raw PCM samples directly. (This call will block if the channel is full.)
                if sample_tx.send(samples.to_vec()).is_err() {
                    log::warn!("Audio output channel closed");
                    break;
                }
            },
            Err(e) => {
                log::warn!("Error decoding packet: {}", e);
            }
        }
        // Small sleep to yield CPU time.
        std::thread::sleep(Duration::from_millis(1));
    }
    log::info!("Exiting decode loop, playback complete");
    Ok(())
}
