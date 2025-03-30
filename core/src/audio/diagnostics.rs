// core/src/audio/diagnostics.rs
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{info, error, warn, debug};
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;

use crate::audio::buffer::AudioRingBuffer;

/// Logs detailed information about audio devices
pub fn log_audio_devices() {
    info!("================ AUDIO DEVICE DIAGNOSTICS ================");
    
    let host = cpal::default_host();
    info!("Host: {}", host.id().name());
    
    // Check all available output devices
    match host.output_devices() {
        Ok(devices) => {
            let devices: Vec<_> = devices.collect();
            info!("Found {} output devices:", devices.len());
            
            for (i, device) in devices.iter().enumerate() {
                let name = device.name().unwrap_or_else(|_| "Unknown".into());
                info!("  Device {}: {}", i, name);
                
                // Log supported output configs
                match device.supported_output_configs() {
                    Ok(configs) => {
                        let configs: Vec<_> = configs.collect();
                        info!("    Supported configs: {}", configs.len());
                        
                        for (j, config) in configs.iter().enumerate() {
                            info!("    Config {}: channels={}, min_rate={}Hz, max_rate={}Hz, sample_format={:?}",
                                  j, 
                                  config.channels(),
                                  config.min_sample_rate().0,
                                  config.max_sample_rate().0,
                                  config.sample_format());
                        }
                    },
                    Err(e) => error!("    Error getting supported configs: {}", e),
                }
            }
            
            // Check default output device
            match host.default_output_device() {
                Some(device) => {
                    let name = device.name().unwrap_or_else(|_| "Unknown".into());
                    info!("Default output device: {}", name);
                },
                None => error!("No default output device available!"),
            }
        },
        Err(e) => error!("Error getting output devices: {}", e),
    }
    
    info!("==========================================================");
}

/// Tests if audio can be played by generating a simple sine wave
pub fn test_audio_output() -> bool {
    info!("================ AUDIO OUTPUT TEST ================");
    
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(device) => device,
        None => {
            error!("No default audio device found!");
            return false;
        }
    };
    
    let name = device.name().unwrap_or_else(|_| "Unknown".into());
    info!("Using device: {}", name);
    
    // Get a supported config
    let config = match device.default_output_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Error getting default output config: {}", e);
            return false;
        }
    };
    
    info!("Using config: channels={}, sample_rate={}Hz, sample_format={:?}",
          config.channels(),
          config.sample_rate().0,
          config.sample_format());
    
    // Variables for sine wave
    let sample_rate = config.sample_rate().0 as f32;
    let frequency = 440.0; // A4 note
    let mut phase = 0.0;
    let success = Arc::new(Mutex::new(false));
    let success_clone = Arc::clone(&success);
    
    // Create and play stream
    let stream = match device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Generate sine wave
            for sample in data.iter_mut() {
                // Simple sine wave
                *sample = 0.5 * (phase * 2.0 * PI).sin();
                phase += frequency / sample_rate;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
            }
            
            // If we got here, mark success
            if let Ok(mut s) = success.lock() {
                *s = true;
            }
        },
        |err| error!("Error during audio playback: {}", err),
        None,
    ) {
        Ok(stream) => stream,
        Err(e) => {
            error!("Error building audio stream: {}", e);
            return false;
        }
    };
    
    // Play for a short time
    match stream.play() {
        Ok(_) => info!("Started audio test playback"),
        Err(e) => {
            error!("Error starting audio stream: {}", e);
            return false;
        }
    }
    
    // Wait a bit for the callback to run
    std::thread::sleep(Duration::from_millis(200));
    
    // Check if we were successful
    let result = match success_clone.lock() {
        Ok(success) => *success,
        Err(_) => false,
    };
    
    info!("Audio test result: {}", if result { "SUCCESS" } else { "FAILED" });
    info!("=================================================");
    
    result
}

/// Validate and debug the audio ring buffer
pub fn test_audio_buffer() {
    info!("================ AUDIO BUFFER TEST ================");
    
    // Create a small buffer for testing
    let mut buffer = AudioRingBuffer::new(1024);
    
    // Test writing to the buffer
    let test_samples = vec![0.5f32; 512];
    let written = buffer.write(&test_samples);
    info!("Wrote {} samples to buffer", written);
    info!("Buffer status: {}/{} samples available", buffer.available(), buffer.capacity());
    
    // Test reading from the buffer
    let mut output = vec![0.0f32; 256];
    let read = buffer.read(&mut output);
    info!("Read {} samples from buffer", read);
    info!("Buffer status: {}/{} samples available", buffer.available(), buffer.capacity());
    
    // Test for data integrity
    let all_match = output.iter().take(read).all(|&s| (s - 0.5).abs() < 0.0001);
    info!("Data integrity test: {}", if all_match { "PASSED" } else { "FAILED" });
    
    info!("=================================================");
}

/// Dumps detailed stats about a ring buffer for debugging
pub fn dump_buffer_stats(buffer: &AudioRingBuffer) {
    info!("AudioRingBuffer stats:");
    info!("  Capacity: {} samples", buffer.capacity());
    info!("  Available: {} samples", buffer.available());
    info!("  Fill level: {:.1}%", 
          (buffer.available() as f32 / buffer.capacity() as f32) * 100.0);
}

/// Save debug info about an audio file
pub fn dump_file_info(path: &str) {
    use ffmpeg_sys_next as ffmpeg;
    use std::ffi::CString;
    
    info!("================ AUDIO FILE INFO ================");
    info!("File: {}", path);
    
    unsafe {
        // Initialize FFmpeg if needed
        ffmpeg::avformat_network_init();
        
        // Open input file
        let c_path = match CString::new(path) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid path: {}", e);
                return;
            }
        };
        
        let mut format_ctx: *mut ffmpeg::AVFormatContext = std::ptr::null_mut();
        let ret = ffmpeg::avformat_open_input(
            &mut format_ctx,
            c_path.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut()
        );
        
        if ret < 0 || format_ctx.is_null() {
            error!("Could not open file: error code {} ({})", ret, ffmpeg_error_string(ret));
            return;
        }
        
        // Get stream info
        let ret = ffmpeg::avformat_find_stream_info(format_ctx, std::ptr::null_mut());
        if ret < 0 {
            error!("Could not find stream info: error code {} ({})", ret, ffmpeg_error_string(ret));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return;
        }
        
        // Log format info
        info!("Format: {}", std::ffi::CStr::from_ptr((*(*format_ctx).iformat).name).to_string_lossy());
        if (*format_ctx).duration > 0 {
            let seconds = (*format_ctx).duration as f64 / ffmpeg::AV_TIME_BASE as f64;
            info!("Duration: {:.2} seconds", seconds);
        } else {
            info!("Duration: unknown");
        }
        
        // Log streams info
        info!("Streams: {}", (*format_ctx).nb_streams);
        for i in 0..(*format_ctx).nb_streams {
            let stream = *(*format_ctx).streams.offset(i as isize);
            let codec_type = (*(*stream).codecpar).codec_type;
            
            // Get stream type name
            let type_name = match codec_type {
                ffmpeg::AVMediaType::AVMEDIA_TYPE_AUDIO => "audio",
                ffmpeg::AVMediaType::AVMEDIA_TYPE_VIDEO => "video",
                _ => "other",
            };
            
            info!("Stream {}: {} stream", i, type_name);
            
            if codec_type == ffmpeg::AVMediaType::AVMEDIA_TYPE_AUDIO {
                // Get audio details
                let sample_rate = (*(*stream).codecpar).sample_rate;
                let channels = (*(*stream).codecpar).ch_layout.nb_channels;
                let codec_id = (*(*stream).codecpar).codec_id;
                
                info!("  Sample rate: {} Hz", sample_rate);
                info!("  Channels: {}", channels);
                
                // Get codec name
                let codec = ffmpeg::avcodec_find_decoder(codec_id);
                if !codec.is_null() {
                    info!("  Codec: {}", std::ffi::CStr::from_ptr((*codec).name).to_string_lossy());
                } else {
                    info!("  Codec ID: {:?}", codec_id);
                }
            }
        }
        
        // Clean up
        ffmpeg::avformat_close_input(&mut format_ctx);
    }
    
    info!("=================================================");
}

/// Debugging function to monitor audio callbacks and detect audio flow issues
pub fn create_diagnostic_stream() -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(device) => device,
        None => {
            error!("No default audio device available for diagnostics");
            return None;
        }
    };
    
    let config = match device.default_output_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Error getting default output config: {}", e);
            return None;
        }
    };
    
    // Create a debug file for audio analysis
    let debug_file = Arc::new(Mutex::new(match File::create("audio_debug.dat") {
        Ok(file) => Some(file),
        Err(e) => {
            error!("Could not create debug file: {}", e);
            None
        }
    }));
    
    let debug_file_clone = Arc::clone(&debug_file);
    let last_log = Arc::new(Mutex::new(Instant::now()));
    let last_log_clone = Arc::clone(&last_log);
    let callback_count = Arc::new(Mutex::new(0u32));
    let callback_count_clone = Arc::clone(&callback_count);
    
    let stream = match device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Count callbacks
            if let Ok(mut count) = callback_count.lock() {
                *count += 1;
            }
            
            // Log stats periodically
            if let Ok(mut last) = last_log.lock() {
                if last.elapsed() > Duration::from_secs(1) {
                    debug!("Audio callback processing {} samples", data.len());
                    
                    // Count non-zero samples as a basic audio check
                    let non_zero = data.iter().filter(|&&s| s.abs() > 0.000001).count();
                    debug!("  Non-zero samples: {}/{} ({:.1}%)", 
                           non_zero, data.len(), 
                           (non_zero as f32 / data.len() as f32) * 100.0);
                    
                    *last = Instant::now();
                }
            }
            
            // Write debug data to file (only every 10th callback to avoid huge files)
            if let Ok(count) = callback_count.lock() {
                if *count % 10 == 0 {
                    if let Ok(file_opt) = debug_file_clone.lock() {
                        if let Some(mut file) = file_opt.as_ref() {
                            // Just write a small sample
                            let samples_to_write = std::cmp::min(data.len(), 100);
                            let _ = file.write_all(
                                &data[..samples_to_write]
                                    .iter()
                                    .map(|&s| (s * 32768.0) as i16)
                                    .flat_map(|s| s.to_le_bytes().to_vec())
                                    .collect::<Vec<u8>>()
                            );
                        }
                    }
                }
            }
        },
        |err| error!("Error in diagnostic audio stream: {}", err),
        None,
    ) {
        Ok(stream) => stream,
        Err(e) => {
            error!("Error building diagnostic audio stream: {}", e);
            return None;
        }
    };
    
    // Start the stream
    if let Err(e) = stream.play() {
        error!("Error starting diagnostic audio stream: {}", e);
        return None;
    }
    
    info!("Started diagnostic audio stream");
    
    // Start a monitoring thread
    std::thread::spawn(move || {
        let mut last_check = Instant::now();
        let mut last_count = 0;
        
        loop {
            std::thread::sleep(Duration::from_secs(5));
            
            // Check callback progress
            if let Ok(count) = callback_count_clone.lock() {
                let elapsed = last_check.elapsed();
                let rate = (*count - last_count) as f32 / elapsed.as_secs_f32();
                
                info!("Audio diagnostics: {} callbacks/sec", rate);
                
                if rate < 1.0 {
                    warn!("Audio callbacks running very slowly or not at all!");
                }
                
                last_count = *count;
                last_check = Instant::now();
            }
            
            // Update log timestamp
            if let Ok(mut last) = last_log_clone.lock() {
                *last = Instant::now();
            }
        }
    });
    
    Some(stream)
}

// FFmpeg error helper function
pub fn ffmpeg_error_string(error_code: i32) -> String {
    unsafe {
        use ffmpeg_sys_next as ffmpeg;
        let mut buffer = vec![0i8; 1024];
        ffmpeg::av_strerror(error_code, buffer.as_mut_ptr(), buffer.len() as usize);
        std::ffi::CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
}