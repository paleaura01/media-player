// core/src/audio/device.rs
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use anyhow::{Result, anyhow};
use log::{info, error};
// We need StreamTrait for the stream.play() method
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio::buffer::AudioRingBuffer;
use crate::audio::position::PlaybackPosition;

/// Sets up audio output device and creates a stream for playback
pub fn setup_audio_device(
    file_sample_rate: u32,
    channel_count: usize,
    _pause_flag: Arc<AtomicBool>,
    _stop_flag: Arc<AtomicBool>,
    _playback_position: Arc<Mutex<PlaybackPosition>>,
    _volume_arc: Arc<Mutex<f32>>
) -> Result<(u32, cpal::Stream)> {
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
        Ok(configs) => configs.filter(|c| c.channels() >= channel_count as u16).collect::<Vec<_>>(),
        Err(e) => {
            error!("Failed to get device configs: {}", e);
            return Err(anyhow!("Failed to get audio device configurations"));
        }
    };
    
    if config_range.is_empty() {
        return Err(anyhow!("No suitable output configuration found for device"));
    }
    
    // Try to match device sample rate with file sample rate
    let desired_sample_rates = [44100, 48000, 96000, 192000];
    let mut selected_config = None;
    
    for &rate in &desired_sample_rates {
        if rate == file_sample_rate {
            for config in &config_range {
                if rate >= config.min_sample_rate().0 && rate <= config.max_sample_rate().0 {
                    selected_config = Some(config.with_sample_rate(cpal::SampleRate(rate)));
                    info!("Selected output sample rate: {} Hz (exact match with file)", rate);
                    break;
                }
            }
        }
        
        if selected_config.is_some() {
            break;
        }
    }
    
    // If no exact match with file rate, prefer 44.1 kHz for most music files
    if selected_config.is_none() {
        for config in &config_range {
            if 44100 >= config.min_sample_rate().0 && 44100 <= config.max_sample_rate().0 {
                selected_config = Some(config.with_sample_rate(cpal::SampleRate(44100)));
                info!("Selected output sample rate: 44100 Hz (preferred rate for music)");
                break;
            }
        }
    }
    
    // If still no match, pick the first available config
    let device_config = selected_config.unwrap_or_else(|| {
        let config = &config_range[0];
        let sample_rate = if file_sample_rate <= config.min_sample_rate().0 {
            config.min_sample_rate().0
        } else if file_sample_rate >= config.max_sample_rate().0 {
            config.max_sample_rate().0
        } else {
            file_sample_rate
        };
        
        info!("Using fallback output sample rate: {} Hz", sample_rate);
        config.with_sample_rate(cpal::SampleRate(sample_rate))
    });
    
    let output_sample_rate = device_config.sample_rate().0;
    let config = device_config.config();
    
    info!("Output device config: {:?}", config);
    
    // Create a ring buffer for audio samples with explicit type annotation
    let buffer_size = ((output_sample_rate as usize * channel_count as usize) / 10).max(1024);
    let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(buffer_size * 4)));
    let ring_buffer_stream: Arc<Mutex<AudioRingBuffer>> = Arc::clone(&ring_buffer);
    
    // Flag to signal when audio needs more data
    let needs_data = Arc::new(AtomicBool::new(true));
    let needs_data_stream = Arc::clone(&needs_data);
    
    info!("Building audio output stream...");
    let stream = match device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // This method returns a Vec of samples and the count as owned values
            let get_samples = || -> (Vec<f32>, usize) {
                // Try to get the ring buffer lock
                if let Ok(mut buffer) = ring_buffer_stream.lock() {
                    // Signal that we need more data if the buffer is low
                    if buffer.available() < buffer_size / 2 {
                        needs_data_stream.store(true, Ordering::Release);
                    }
                    
                    // Prepare a buffer to hold samples
                    let mut samples = vec![0.0; data.len()];
                    // Read samples from the ring buffer into our local buffer
                    let count = buffer.read(&mut samples);
                    
                    // Return owned values, not references
                    (samples, count)
                } else {
                    // If we couldn't get the lock, return empty data
                    (Vec::new(), 0)
                }
            };
            
            // Use catch_unwind ONLY around the function that returns owned values
            let result = std::panic::catch_unwind(|| {
                get_samples()
            });
            
            // Process the result safely outside the unwind boundary
            match result {
                Ok((samples, count)) => {
                    // Handle the results safely
                    if count > 0 && count <= data.len() && count <= samples.len() {
                        // Copy the samples to the output buffer
                        data[..count].copy_from_slice(&samples[..count]);
                    }
                    
                    // Fill the rest with silence
                    if count < data.len() {
                        for sample in &mut data[count..] {
                            *sample = 0.0;
                        }
                    }
                },
                Err(_) => {
                    // If there was a panic, fill with silence
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                }
            }
        },
        |err| { error!("Audio output error: {}", err); },
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to build audio stream: {}", e);
            return Err(anyhow!("Failed to build audio stream: {}", e));
        }
    };
    
    Ok((output_sample_rate, stream))
}