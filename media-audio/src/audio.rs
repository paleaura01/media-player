use std::fs::File;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}, Mutex};
use std::time::Duration;

use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub fn init() -> Result<()> {
    log::info!("Audio subsystem initialized");
    Ok(())
}

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

    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| anyhow!("No output audio device available"))?;
    
    let supported_configs = device.supported_output_configs()?;
    let device_config = supported_configs
        .filter(|cfg| cfg.channels() >= channel_count as u16)
        .find(|cfg| {
            let min = cfg.min_sample_rate().0;
            let max = cfg.max_sample_rate().0;
            file_sample_rate >= min && file_sample_rate <= max
        })
        .map(|cfg| cfg.with_sample_rate(cpal::SampleRate(file_sample_rate)))
        .ok_or_else(|| anyhow!("No suitable audio output config found"))?;
        
    let config = device_config.config();
    log::info!("Output device: {} with config: {:?}", 
        device.name().unwrap_or_else(|_| "Unknown".to_string()), config);

    if file_sample_rate != config.sample_rate.0 {
        log::warn!("File sample rate {} Hz does not match output device rate {} Hz.",
            file_sample_rate, config.sample_rate.0);
    }

    let buffer_capacity = file_sample_rate as usize * channel_count as usize;
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(buffer_capacity)));
    let audio_buffer_stream = Arc::clone(&audio_buffer);
    let samples_in_buffer = Arc::new(AtomicUsize::new(0));
    let samples_in_buffer_stream = Arc::clone(&samples_in_buffer);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut buffer = audio_buffer_stream.lock().unwrap();
            let samples_needed = data.len();
            let samples_available = samples_in_buffer_stream.load(Ordering::Acquire);
            
            if samples_available == 0 {
                for sample in data.iter_mut() {
                    *sample = 0.0;
                }
                return;
            }
            
            let samples_to_copy = std::cmp::min(samples_needed, samples_available);
            data[..samples_to_copy].copy_from_slice(&buffer[..samples_to_copy]);
            
            if samples_to_copy < buffer.len() {
                buffer.copy_within(samples_to_copy.., 0);
            }
            
            buffer.truncate(samples_available - samples_to_copy);
            samples_in_buffer_stream.store(buffer.len(), Ordering::Release);
            
            if samples_to_copy < samples_needed {
                for sample in &mut data[samples_to_copy..] {
                    *sample = 0.0;
                }
            }
        },
        |err| {
            log::error!("Audio output error: {}", err);
        },
        None,
    )?;
    
    stream.play()?;
    log::info!("Audio stream started");

    let spec = SignalSpec::new(file_sample_rate, channels);
    let mut sample_buf = SampleBuffer::<f32>::new(4096, spec);
    
    log::info!("Starting decode loop");
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            log::info!("Stop signal received; exiting decode loop");
            break;
        }
        
        if pause_flag.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(20));
            continue;
        }

        let current_samples = samples_in_buffer.load(Ordering::Acquire);
        if current_samples >= buffer_capacity - 4096 {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        let packet = match format.next_packet() {
            Ok(packet) => {
                if packet.track_id() != track_id {
                    continue;
                }
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

        match decoder.decode(&packet) {
            Ok(decoded) => {
                sample_buf.copy_interleaved_ref(decoded);
                let samples = sample_buf.samples();
                let mut buffer = audio_buffer.lock().unwrap();
                let current_len = buffer.len();
                let samples_to_add = samples.len();
                
                if current_len + samples_to_add <= buffer_capacity {
                    buffer.extend_from_slice(samples);
                    samples_in_buffer.store(current_len + samples_to_add, Ordering::Release);
                } else {
                    log::warn!("Buffer overflow prevented - dropping samples");
                }
            },
            Err(e) => {
                log::warn!("Error decoding packet: {}", e);
            }
        }
    }
    
    log::info!("Decoding complete");
    Ok(())
}
