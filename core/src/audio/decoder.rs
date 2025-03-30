// core/src/audio/decoder.rs
use std::path::{Path, PathBuf};  // Added PathBuf import here
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ffmpeg_sys_next as ffmpeg;
use ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_AUDIO;
use ffmpeg_sys_next::AVSampleFormat::AV_SAMPLE_FMT_FLT;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use crate::audio::buffer::AudioRingBuffer;
use crate::audio::position::PlaybackPosition;
use crate::PlayerState;

// Initialize FFmpeg only once
static mut FFMPEG_INITIALIZED: bool = false;

// Define constants for buffer safety
const MAX_CHANNELS: usize = 8;
const MAX_BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16MB maximum buffer size
const MAX_DIRECTORY_DEPTH: usize = 20; // Maximum directory recursion depth

// Helper function to convert C string to Rust string
unsafe fn to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

pub fn initialize_ffmpeg() -> Result<()> {
    unsafe {
        if !FFMPEG_INITIALIZED {
            ffmpeg::avformat_network_init();
            FFMPEG_INITIALIZED = true;
            info!("FFmpeg initialized successfully");
        }
    }
    Ok(())
}

// Get list of supported extensions for UI filtering
pub fn get_supported_extensions() -> Vec<String> {
    // Comprehensive list of audio formats supported by FFmpeg
    let common_extensions = [
        // Common formats
        "mp3", "wav", "flac", "ogg", "m4a", "aac", "opus", "wma", "ape", "mka",
        "mp4", "mp2", "ac3", "amr", "au", "mid", "midi", "ra", "rm", "tta", "wv", 
        "caf", "aiff", "aif",
        
        // Less common but supported formats
        "oga", "m4b", "dts", "mpc", "tak", "pcm", "sbc", "voc", "w64", "webm",
        "3ga", "dsf", "dff", "gsm", "spx", "shn", "xa", "svx", "8svx", "pvf",
        "sf", "vox", "iff", "sln", "aa3", "oma", "at3", "adx", "adp", "dxa",
        "dca", "imc", "wady", "mat", "mmf", "eam", "eas", "paf", "raw",
        
        // Module formats
        "mod", "s3m", "xm", "it",
        
        // Additional container formats that might contain audio
        "mkv", "avi", "mov", "wmv", "3gp", "ogv", "mka",
        
        // Additional variations and capitalized extensions (Windows compatibility)
        "MP3", "WAV", "FLAC", "OGG", "M4A", "AAC", "OPUS", "WMA", "APE", "MKA",
        "MP4", "MP2", "AC3", "AMR", "AU", "MID", "MIDI", "RA", "RM", "TTA", "WV",
        "CAF", "AIFF", "AIF"
    ];
    
    common_extensions.iter().map(|&s| s.to_string()).collect()
}

// Check if a file is supported by FFmpeg
pub fn is_supported_audio_format(path: &str) -> bool {
    // Initialize FFmpeg if needed
    if let Err(_) = initialize_ffmpeg() {
        return false;
    }
    
    // Special handling for UNC paths
    let normalized_path = if path.starts_with("\\\\?\\UNC\\") {
        // Convert Windows long path format to regular UNC path for checks
        let unc_path = format!("\\\\{}", &path[8..]);
        unc_path
    } else if path.starts_with("\\\\?\\") {
        // Strip the long path prefix for local files
        path[4..].to_string()
    } else {
        path.to_string()
    };
    
    // For network paths, assume supported based on extension
    if normalized_path.starts_with("\\\\") || normalized_path.contains("://") {
        let lowercase_path = normalized_path.to_lowercase();
        return get_supported_extensions().iter().any(|ext| lowercase_path.ends_with(&format!(".{}", ext)));
    }
    
    // Check if local file exists
    if !Path::new(&normalized_path).exists() {
        return false;
    }
    
    // Try to open the file with FFmpeg
    unsafe {
        let c_path = match CString::new(normalized_path) {
            Ok(s) => s,
            Err(_) => return false,
        };
        
        let mut format_ctx: *mut ffmpeg::AVFormatContext = std::ptr::null_mut();
        
        // Try to open the file
        let ret = ffmpeg::avformat_open_input(
            &mut format_ctx,
            c_path.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        
        if ret < 0 || format_ctx.is_null() {
            return false;
        }
        
        // Find stream info
        let ret = ffmpeg::avformat_find_stream_info(format_ctx, std::ptr::null_mut());
        if ret < 0 {
            ffmpeg::avformat_close_input(&mut format_ctx);
            return false;
        }
        
        // Check for audio stream
        let mut has_audio = false;
        for i in 0..(*format_ctx).nb_streams {
            let stream = *(*format_ctx).streams.offset(i as isize);
            let codec_type = (*(*stream).codecpar).codec_type;
            
            if codec_type == AVMEDIA_TYPE_AUDIO {
                has_audio = true;
                break;
            }
        }
        
        // Clean up
        ffmpeg::avformat_close_input(&mut format_ctx);
        
        has_audio
    }
}

// Identify network paths
pub fn is_network_path(path: &str) -> bool {
    path.starts_with("\\\\") || 
    path.starts_with("\\\\?\\UNC\\") || 
    path.contains("://")
}

// Normal path handling
fn normalize_path_for_check(path: &str) -> String {
    if path.starts_with("\\\\?\\UNC\\") {
        return format!("\\\\{}", &path[8..]);
    } else if path.starts_with("\\\\?\\") {
        return path[4..].to_string();
    }
    path.to_string()
}

// Safe audio frame processing for regular playback
unsafe fn process_audio_frame_safe(
    frame: *mut ffmpeg::AVFrame,
    swr_ctx: *mut ffmpeg::SwrContext,
    channel_count: usize,
    output_sample_rate: u32,
    file_sample_rate: u32,
    volume_arc: Arc<Mutex<f32>>,
    ring_buffer: Arc<Mutex<AudioRingBuffer>>,
    needs_data: &Arc<AtomicBool>,
    last_buffer_warn: &mut Instant
) -> u64 {
    if frame.is_null() {
        return 0;
    }
    
    let nb_samples = (*frame).nb_samples;
    if nb_samples <= 0 {
        warn!("Skipping empty frame: {} samples", nb_samples);
        return 0;
    }
    
    // Calculate output samples directly with the correct ratio
    let ratio = output_sample_rate as f64 / file_sample_rate as f64;
    let output_samples_calc = (nb_samples as f64 * ratio).ceil() as i32;
    
    // Log sample rates periodically
    static mut LAST_RATE_LOG: Option<Instant> = None;
    let rate_log_needed = unsafe {
        match LAST_RATE_LOG {
            None => {
                LAST_RATE_LOG = Some(Instant::now());
                true
            },
            Some(last) => {
                let elapsed = Instant::now().duration_since(last);
                if elapsed > Duration::from_secs(5) {
                    LAST_RATE_LOG = Some(Instant::now());
                    true
                } else {
                    false
                }
            }
        }
    };
    
    if rate_log_needed {
        info!("Resampling from {}Hz to {}Hz (ratio: {:.2}), frame size: {} samples", 
              file_sample_rate, output_sample_rate, ratio, nb_samples);
    }
    
    // Allocate properly sized output buffer - directly calculate the right size
    let mut output: Vec<u8> = Vec::new();
    let dest_nb_samples = output_samples_calc;
    let dest_data_size = ffmpeg::av_samples_get_buffer_size(
        std::ptr::null_mut(),
        channel_count as i32,
        dest_nb_samples,
        AV_SAMPLE_FMT_FLT,
        1
    );
    
    if dest_data_size <= 0 {
        error!("Invalid destination buffer size calculation: {}", dest_data_size);
        return 0;
    }
    
    // Safely allocate the right sized buffer on the heap
    output.resize(dest_data_size as usize, 0);
    let mut output_ptr = output.as_mut_ptr();
    
    // Create a vector of pointers to each channel's data
    let mut output_ptrs: Vec<*mut u8> = Vec::with_capacity(channel_count);
    for _ in 0..channel_count {
        output_ptrs.push(output_ptr);
        // Point to the next channel's position
        output_ptr = output_ptr.add(dest_nb_samples as usize * std::mem::size_of::<f32>());
    }
    
    // Perform the resampling
    let out_samples = ffmpeg::swr_convert(
        swr_ctx,
        output_ptrs.as_mut_ptr(),
        dest_nb_samples,
        (*frame).extended_data as *mut *const u8,
        nb_samples
    );
    
    if out_samples <= 0 {
        error!("Resampling failed: {}", out_samples);
        return 0;
    }
    
    // Convert the raw pointer to our output buffer to a slice of f32 samples
    let output_samples = std::slice::from_raw_parts(
        output.as_ptr() as *const f32,
        (out_samples as usize) * channel_count
    );
    
    // Apply volume
    let volume = match volume_arc.lock() {
        Ok(v) => *v,
        Err(_) => 1.0,
    };
    
    // Create a new volume-adjusted buffer
    let mut volume_adjusted = Vec::with_capacity(output_samples.len());
    for &sample in output_samples {
        volume_adjusted.push(sample * volume);
    }
    
    // Write to ring buffer
    let buffer_health = {
        if let Ok(mut rb) = ring_buffer.lock() {
            let _written = rb.write_safe(&volume_adjusted);
            
            let available = rb.available();
            let buffer_size = rb.capacity();
            if buffer_size > 0 {
                (available as f32) / (buffer_size as f32)
            } else {
                0.0
            }
        } else {
            0.0
        }
    };
    
    // Reset needs_data flag if we've processed enough data
    if buffer_health > 0.5 {
        needs_data.store(false, Ordering::Release);
    }
    
    *last_buffer_warn = Instant::now();
    
    // Return the number of frames decoded
    nb_samples as u64
}

pub fn play_audio_file(
    path: &str,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    state_arc: Arc<Mutex<PlayerState>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume_arc: Arc<Mutex<f32>>,
) -> Result<()> {
    info!("Attempting to play file: {}", path);
    
    // Initialize FFmpeg
    initialize_ffmpeg()?;
    
    // Check if file exists for local files
    let is_network = is_network_path(path);
    if !is_network {
        let norm_path = normalize_path_for_check(path);
        if !Path::new(&norm_path).exists() {
            error!("File not found: {}", norm_path);
            return Err(anyhow!("File not found: {}", norm_path));
        }
    }
    
    // Set streaming mode
    if let Ok(mut state) = state_arc.lock() {
        state.network_buffering = false;
        state.buffer_progress = 1.0;
    }
    
    // Variables for metadata
    let mut track_duration_secs = 300.0; // Default
    let mut channel_count = 2; // Default
    let mut sample_rate = 44100; // Default

    unsafe {
        // Create a C-string from the path
        let c_path = match CString::new(path) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid path string: {}", e);
                return Err(anyhow!("Invalid path string: {}", e));
            }
        };
        
        // Create format context
        let mut format_ctx: *mut ffmpeg::AVFormatContext = std::ptr::null_mut();
        
        // For network files, create format options with longer timeouts
        let mut options: *mut ffmpeg::AVDictionary = std::ptr::null_mut();
        if is_network {
            // Set timeout values
            let timeout_key = CString::new("timeout").unwrap();
            let timeout_val = CString::new("10000000").unwrap(); // 10 seconds in microseconds
            
            ffmpeg::av_dict_set(&mut options, timeout_key.as_ptr(), timeout_val.as_ptr(), 0);
        }
        
        // Open input
        let ret = ffmpeg::avformat_open_input(
            &mut format_ctx,
            c_path.as_ptr(),
            std::ptr::null_mut(),
            &mut options
        );
        
        // Free options dictionary
        if !options.is_null() {
            ffmpeg::av_dict_free(&mut options);
        }
        
        if ret < 0 || format_ctx.is_null() {
            let error_buf = [0i8; 1024];
            ffmpeg::av_strerror(ret, error_buf.as_ptr() as *mut i8, 1024);
            let error_msg = to_string(error_buf.as_ptr());
            error!("Could not open input file: {} ({})", error_msg, ret);
            return Err(anyhow!("Could not open input file: {}", error_msg));
        }
        
        // Find stream info
        let ret = ffmpeg::avformat_find_stream_info(format_ctx, std::ptr::null_mut());
        if ret < 0 {
            let error_buf = [0i8; 1024];
            ffmpeg::av_strerror(ret, error_buf.as_ptr() as *mut i8, 1024);
            let error_msg = to_string(error_buf.as_ptr());
            ffmpeg::avformat_close_input(&mut format_ctx);
            error!("Could not find stream information: {} ({})", error_msg, ret);
            return Err(anyhow!("Could not find stream information: {}", error_msg));
        }
        
        // Find audio stream
        let mut audio_stream_idx: i32 = -1;
        
        for i in 0..(*format_ctx).nb_streams {
            let stream = *(*format_ctx).streams.offset(i as isize);
            let codec_params = (*stream).codecpar;
            
            if (*codec_params).codec_type == AVMEDIA_TYPE_AUDIO {
                audio_stream_idx = i as i32;
                channel_count = (*codec_params).ch_layout.nb_channels as usize;
                sample_rate = (*codec_params).sample_rate as u32;
                break;
            }
        }
        
        if audio_stream_idx == -1 {
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not find any audio stream in the file"));
        }
        
        // Limit channel count for safety
        let safe_channel_count = std::cmp::min(channel_count, MAX_CHANNELS);
        if channel_count > MAX_CHANNELS {
            warn!("Limiting channels from {} to {} for safety", channel_count, safe_channel_count);
            channel_count = safe_channel_count;
        }
        
        info!("Found audio stream: {} channels, {} Hz", channel_count, sample_rate);
        
        // Get the stream for codec info
        let stream = *(*format_ctx).streams.offset(audio_stream_idx as isize);
        let codec_params = (*stream).codecpar;
        
        // Find decoder
        let codec = ffmpeg::avcodec_find_decoder((*codec_params).codec_id);
        if codec.is_null() {
            error!("Unsupported codec ID: {:?}", (*codec_params).codec_id);
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Unsupported codec"));
        }
        
        info!("Using codec: {}", CStr::from_ptr((*codec).name).to_string_lossy());
        
        // Create codec context
        let codec_ctx = ffmpeg::avcodec_alloc_context3(codec);
        if codec_ctx.is_null() {
            error!("Failed to allocate codec context");
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not allocate codec context"));
        }
        
        // Copy codec parameters to context
        if ffmpeg::avcodec_parameters_to_context(codec_ctx, codec_params) < 0 {
            error!("Failed to copy codec parameters to context");
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not copy codec parameters to context"));
        }
        
        // Force safe channel count in codec context
        (*codec_ctx).ch_layout.nb_channels = safe_channel_count as c_int;
        
        // Log codec parameters for debugging
        info!("Codec parameters: sample_fmt={:?}, sample_rate={}, channels={}", 
            (*codec_ctx).sample_fmt,
            (*codec_ctx).sample_rate,
            (*codec_ctx).ch_layout.nb_channels);
        
        // Open codec
        if ffmpeg::avcodec_open2(codec_ctx, codec, std::ptr::null_mut()) < 0 {
            error!("Could not open codec");
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not open codec"));
        }
        
        // Calculate duration from format context
        if (*format_ctx).duration > 0 {
            track_duration_secs = (*format_ctx).duration as f64 / ffmpeg::AV_TIME_BASE as f64;
        }
        
        let track_duration = Duration::from_secs_f64(track_duration_secs);
        info!("Track duration: {:?}", track_duration);
        
        // Update player state with duration
        if let Ok(mut state) = state_arc.lock() {
            state.duration = Some(track_duration);
        }
        
        // Set up audio output with cpal
        info!("Setting up audio output with CPAL...");
        let host = cpal::default_host();
        info!("Audio host: {}", host.id().name());
        
        let device = match host.default_output_device() {
            Some(device) => {
                info!("Using output device: {}", device.name().unwrap_or_else(|_| String::from("Unknown")));
                device
            },
            None => {
                error!("No output audio device available!");
                return Err(anyhow!("No output audio device available"));
            }
        };
    
        let mut config_range = device
            .supported_output_configs()
            .map_err(|e| {
                error!("Failed to get device configs: {}", e);
                anyhow!("Failed to get device configs: {}", e)
            })?
            .filter(|c| c.channels() >= channel_count as u16)
            .collect::<Vec<_>>();
        
        if config_range.is_empty() {
            error!("No suitable output config found for device (needed {} channels)", channel_count);
            return Err(anyhow!("No suitable output config found for device"));
        }
        
        config_range.sort_by_key(|c| c.min_sample_rate().0);
        let desired_sample_rates = [sample_rate, 48000, 44100, 96000, 192000];
        let mut selected_config = None;
    
        // Try to find a config that supports our desired sample rates
        for &rate in &desired_sample_rates {
            for c in &config_range {
                if rate >= c.min_sample_rate().0 && rate <= c.max_sample_rate().0 {
                    selected_config = Some(c.with_sample_rate(cpal::SampleRate(rate)));
                    info!("Selected output config: {} channels, {} Hz", 
                          c.channels(), rate);
                    break;
                }
            }
            if selected_config.is_some() {
                break;
            }
        }
    
        let device_config = selected_config.unwrap_or_else(|| {
            let config = &config_range[0];
            
            // Choose a safe sample rate close to the original
            let mut target_rate = sample_rate;
            
            // If the file rate is extremely low or high, use a standard rate instead
            if sample_rate < 8000 || sample_rate > 192000 {
                target_rate = 44100; // Use a standard rate
                warn!("File has unusual sample rate ({}Hz). Using standard 44.1kHz output instead.", 
                     sample_rate);
            }
            
            let sample_rate = if target_rate <= config.min_sample_rate().0 {
                config.min_sample_rate().0
            } else if target_rate >= config.max_sample_rate().0 {
                config.max_sample_rate().0
            } else {
                target_rate
            };
            
            info!("Using output sample rate: {} Hz", sample_rate);
            config.clone().with_sample_rate(cpal::SampleRate(sample_rate))
        });
        
        let config = device_config.config();
        let output_sample_rate = config.sample_rate.0;
        info!("Output config: {} channels, {} Hz", config.channels, output_sample_rate);
    
        // Calculate total samples based on duration
        let total_samples = (track_duration_secs * sample_rate as f64) as u64 * channel_count as u64;
        
        if let Ok(mut pos) = playback_position.lock() {
            pos.set_total_samples(total_samples);
            pos.set_channel_count(channel_count);
            pos.sample_rate = sample_rate;
        }
        
        // Set up ring buffer for audio output - make it larger for safety but with limits
        let desired_buffer_frames = (output_sample_rate as usize * channel_count * 4) / 5; // 800ms buffer
        let max_buffer_frames = MAX_BUFFER_SIZE / (channel_count * std::mem::size_of::<f32>());
        let ring_buffer_size = std::cmp::min(desired_buffer_frames, max_buffer_frames);
        
        info!("Creating ring buffer with {} samples ({:.2} MB)", 
              ring_buffer_size,
              (ring_buffer_size * std::mem::size_of::<f32>()) as f32 / (1024.0 * 1024.0));
              
        let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(ring_buffer_size)));
        let ring_buffer_stream = Arc::clone(&ring_buffer);
        
        let needs_data = Arc::new(AtomicBool::new(true));
        let needs_data_stream = Arc::clone(&needs_data);
        
        // Debug the buffer
        if let Ok(rb) = ring_buffer.lock() {
            info!("Ring buffer initialized: capacity={}, available={}",
                  rb.capacity(), rb.available());
        }
    
        // Set up audio output stream
        info!("Building audio output stream...");
        let stream_result = device.build_output_stream(
            &config,
            move |data: &mut [f32], _info| {
                // This callback is run by the audio system when it needs more samples
                let start_time = std::time::Instant::now();
                
                // Read from our ring buffer with additional safety
                let mut samples_read = 0;
                if let Ok(mut rb) = ring_buffer_stream.lock() {
                    // Only read what fits in the output buffer
                    samples_read = rb.read(data);
                    
                    // Explicitly fill the rest with silence to avoid using uninitialized memory
                    if samples_read < data.len() {
                        for s in &mut data[samples_read..] {
                            *s = 0.0;
                        }
                    }
                    
                    // Signal that we need more data if the buffer is getting low
                    if rb.available() < ring_buffer_size / 4 {
                        needs_data_stream.store(true, Ordering::Release);
                    }
                } else {
                    // On error, ensure the entire buffer is zeroed
                    for s in data.iter_mut() {
                        *s = 0.0;
                    }
                    error!("Failed to lock ring buffer in audio callback");
                }
                
                // Log details to help diagnose problems
                if samples_read == 0 {
                    debug!("Audio callback: no samples available - outputting silence");
                }
                
                // Periodically log statistics about the audio callback
                static LAST_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                    
                let last = LAST_LOG.load(std::sync::atomic::Ordering::Relaxed);
                if now > last + 5 {
                    LAST_LOG.store(now, std::sync::atomic::Ordering::Relaxed);
                    
                    // Calculate callback duration
                    let elapsed = start_time.elapsed();
                    
                    // Count non-zero samples as audio presence check
                    let non_zero_samples = data.iter().filter(|&&s| s.abs() > 0.001).count();
                    
                    debug!(
                        "Audio callback: {} samples ({}ms), read {}, non-zero: {} ({}%)",
                        data.len(),
                        elapsed.as_micros() as f64 / 1000.0,
                        samples_read,
                        non_zero_samples,
                        non_zero_samples as f64 * 100.0 / data.len() as f64
                    );
                }
            },
            |err| {
                error!("Audio output error: {}", err);
            },
            None,
        );
        
        let audio_stream = match stream_result {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to build audio stream: {}", e);
                return Err(anyhow!("Failed to start audio: {}", e));
            }
        };
        
        match audio_stream.play() {
            Ok(_) => info!("Started audio playback stream"),
            Err(e) => {
                error!("Failed to start audio stream: {}", e);
                return Err(anyhow!("Failed to start audio: {}", e));
            }
        }
        
        // Create SwrContext for resampling
        let mut swr_ctx: *mut ffmpeg::SwrContext = ffmpeg::swr_alloc();
        
        // Create a new channel layout structure for input
        let mut in_ch_layout = std::mem::zeroed::<ffmpeg::AVChannelLayout>();
        ffmpeg::av_channel_layout_default(&mut in_ch_layout, channel_count as c_int);
        
        // Create a new channel layout structure for output
        let mut out_ch_layout = std::mem::zeroed::<ffmpeg::AVChannelLayout>();
        ffmpeg::av_channel_layout_default(&mut out_ch_layout, channel_count as c_int);
        
        // Set up resampler with new API
        if swr_ctx.is_null() {
            error!("Failed to allocate resampler context");
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to allocate resampler context"));
        }
        
        // Log resampling parameters
        info!("Setting up resampler: {}Hz -> {}Hz, {} channels", 
             sample_rate, output_sample_rate, channel_count);
        
        // Set options on the SwrContext
        ffmpeg::av_opt_set_int(
            swr_ctx as *mut _, 
            CString::new("in_sample_rate")?.as_ptr(), 
            sample_rate as i64, 
            0
        );
        
        ffmpeg::av_opt_set_int(
            swr_ctx as *mut _, 
            CString::new("out_sample_rate")?.as_ptr(), 
            output_sample_rate as i64, 
            0
        );
        
        ffmpeg::av_opt_set_sample_fmt(
            swr_ctx as *mut _, 
            CString::new("in_sample_fmt")?.as_ptr(), 
            (*codec_ctx).sample_fmt, 
            0
        );
        
        ffmpeg::av_opt_set_sample_fmt(
            swr_ctx as *mut _, 
            CString::new("out_sample_fmt")?.as_ptr(), 
            AV_SAMPLE_FMT_FLT, 
            0
        );
        
        // Set channel layouts
        ffmpeg::av_opt_set_chlayout(
            swr_ctx as *mut _,
            CString::new("in_chlayout")?.as_ptr(),
            &in_ch_layout,
            0
        );
                               
        ffmpeg::av_opt_set_chlayout(
            swr_ctx as *mut _,
            CString::new("out_chlayout")?.as_ptr(),
            &out_ch_layout,
            0
        );
        
        // Initialize the resampler
        let swr_init_result = ffmpeg::swr_init(swr_ctx);
        if swr_init_result < 0 {
            let error_buf = [0i8; 1024];
            ffmpeg::av_strerror(swr_init_result, error_buf.as_ptr() as *mut i8, 1024);
            let error_msg = to_string(error_buf.as_ptr());
            error!("Failed to initialize resampler: {} ({})", error_msg, swr_init_result);
            ffmpeg::swr_free(&mut swr_ctx);
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to initialize resampler: {}", error_msg));
        }
        
        // Allocate packet and frame
        let packet = ffmpeg::av_packet_alloc();
        if packet.is_null() {
            error!("Failed to allocate packet");
            ffmpeg::swr_free(&mut swr_ctx);
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to allocate packet"));
        }
        
        let frame = ffmpeg::av_frame_alloc();
        if frame.is_null() {
            error!("Failed to allocate frame");
            ffmpeg::av_packet_free(&mut (packet as *mut _));
            ffmpeg::swr_free(&mut swr_ctx);
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to allocate frame"));
        }
        
        info!("Beginning decode loop for file: {}", path);
        
        // Main decoding loop
        let mut current_frames: u64 = 0;
        let mut is_eof = false;
        let mut last_progress_log = Instant::now();
        let mut last_buffer_warn = Instant::now();
        
        while !is_eof && !stop_flag.load(Ordering::SeqCst) {
            // Handle pause state
            if pause_flag.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            
            // Handle seek requests
            let mut seek_requested = false;
            let mut target_fraction = 0.0;
            
            if let Ok(pos_lock) = playback_position.lock() {
                if let Some(req_flag) = &pos_lock.seek_requested {
                    if req_flag.swap(false, Ordering::SeqCst) {
                        seek_requested = true;
                        if let Some(tgt) = &pos_lock.seek_target {
                            if let Ok(tgt_val) = tgt.lock() {
                                target_fraction = *tgt_val;
                            }
                        }
                    }
                }
            }
    
            if seek_requested {
                info!("Seek requested to position {:.4}", target_fraction);
                
                // Calculate seek position in seconds and convert to stream timebase
                let target_time_seconds = target_fraction * track_duration_secs as f32;
                let timestamp = (target_time_seconds * (*stream).time_base.den as f32 / 
                                (*stream).time_base.num as f32) as i64;
                
                // Flush buffers
                ffmpeg::avcodec_flush_buffers(codec_ctx);
                
                // Perform seek
                let seek_flags = ffmpeg::AVSEEK_FLAG_BACKWARD;
                let ret = ffmpeg::av_seek_frame(
                    format_ctx, 
                    audio_stream_idx,
                    timestamp,
                    seek_flags
                );
                
                if ret < 0 {
                    let error_buf = [0i8; 1024];
                    ffmpeg::av_strerror(ret, error_buf.as_ptr() as *mut i8, 1024);
                    let error_msg = to_string(error_buf.as_ptr());
                    warn!("Seeking failed: {} ({})", error_msg, ret);
                } else {
                    info!("Seek successful");
                    
                    // Update current position
                    let new_frames = (target_fraction * total_samples as f32 / channel_count as f32) as u64;
                    current_frames = new_frames;
                    
                    if let Ok(pos) = playback_position.lock() {
                        let frame_pos = current_frames as usize;
                        pos.set_current_frame(frame_pos);
                    }
                }
                
                // Clear the ring buffer
                if let Ok(mut rb) = ring_buffer.lock() {
                    debug!("Clearing ring buffer during seek");
                    rb.clear();
                }
                
                needs_data.store(true, Ordering::Release);
                continue;
            }
    
            if !needs_data.load(Ordering::Acquire) {
                // If we don't need data yet, sleep briefly
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            
            // Read packet
            let ret = ffmpeg::av_read_frame(format_ctx, packet);
            if ret < 0 {
                if ret == ffmpeg::AVERROR_EOF || ret == ffmpeg::AVERROR(libc::EAGAIN) {
                    info!("End of file reached");
                    is_eof = true;
                    
                    // Signal track completion
                    if let Ok(mut st) = state_arc.lock() {
                        st.track_completed = true;
                    }
                } else {
                    let error_buf = [0i8; 1024];
                    ffmpeg::av_strerror(ret, error_buf.as_ptr() as *mut i8, 1024);
                    let error_msg = to_string(error_buf.as_ptr());
                    warn!("Error reading frame: {} ({})", error_msg, ret);
                }
                continue;
            }
            
            // Skip non-audio packets
            if (*packet).stream_index != audio_stream_idx {
                ffmpeg::av_packet_unref(packet);
                continue;
            }
            
            // Send packet to decoder
            let ret = ffmpeg::avcodec_send_packet(codec_ctx, packet);
            ffmpeg::av_packet_unref(packet);
            
            if ret < 0 {
                let error_buf = [0i8; 1024];
                ffmpeg::av_strerror(ret, error_buf.as_ptr() as *mut i8, 1024);
                let error_msg = to_string(error_buf.as_ptr());
                warn!("Error sending packet to decoder: {} ({})", error_msg, ret);
                continue;
            }
            
            // Process decoded frames
            let mut got_frame = false;
            
            // Limit the number of frames processed per iteration to avoid stack overflow
            let max_frames_per_packet = 4; // Lower this value for safer processing
            let mut frames_processed = 0;
            
            while !got_frame && frames_processed < max_frames_per_packet {
                frames_processed += 1;
                
                let ret = ffmpeg::avcodec_receive_frame(codec_ctx, frame);
                
                if ret == ffmpeg::AVERROR(libc::EAGAIN) || ret == ffmpeg::AVERROR_EOF {
                    break;
                } else if ret < 0 {
                    let error_buf = [0i8; 1024];
                    ffmpeg::av_strerror(ret, error_buf.as_ptr() as *mut i8, 1024);
                    let error_msg = to_string(error_buf.as_ptr());
                    warn!("Error receiving frame from decoder: {} ({})", error_msg, ret);
                    break;
                }
                
                got_frame = true;
                
                // Process the audio frame with enhanced safety
                let frames_decoded = process_audio_frame_safe(
                    frame,
                    swr_ctx, 
                    channel_count,
                    output_sample_rate,
                    sample_rate,
                    volume_arc.clone(),
                    ring_buffer.clone(),
                    &needs_data,
                    &mut last_buffer_warn
                );
                
                // Update position
                if frames_decoded > 0 {
                    current_frames += frames_decoded;
                    if let Ok(pos) = playback_position.lock() {
                        pos.update_current_sample(frames_decoded as usize);
                    }
                }
                
                ffmpeg::av_frame_unref(frame);
            }
            
            // Periodically log progress
            if last_progress_log.elapsed() >= Duration::from_secs(1) {
                let cur_seconds = current_frames as f64 / sample_rate as f64;
                debug!("Playback progress: {:.1}s / {:.1}s ({:.1}%)",
                       cur_seconds,
                       track_duration_secs,
                       (cur_seconds / track_duration_secs) * 100.0);
                       
                // Check ring buffer stats
                if let Ok(rb) = ring_buffer.lock() {
                    debug!("Ring buffer: {}/{} samples available ({:.1}%)",
                           rb.available(),
                           rb.capacity(),
                           rb.available() as f64 * 100.0 / rb.capacity() as f64);
                }
                       
                last_progress_log = Instant::now();
            }
            
            // Sleep a tiny bit to avoid busy-waiting and reduce CPU usage
            thread::sleep(Duration::from_micros(100));
        }
        
        // Cleanup
        info!("Playback complete, cleaning up resources");
        ffmpeg::av_frame_free(&mut (frame as *mut _));
        ffmpeg::av_packet_free(&mut (packet as *mut _));
        ffmpeg::swr_free(&mut swr_ctx);
        ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
        ffmpeg::avformat_close_input(&mut format_ctx);
    }
    
    Ok(())
}

// Enhanced version that accepts additional parameters
pub fn play_audio_file_enhanced(
    path: &str,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    state_arc: Arc<Mutex<PlayerState>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume_arc: Arc<Mutex<f32>>,
    prefetch_mode: bool,  
    buffer_size: Option<usize>,  
) -> Result<()> {
    info!("Playing audio file with enhanced mode - prefetch={}, buffer_size={:?}", 
          prefetch_mode, buffer_size);
          
    // For network paths, we'll apply special handling
    let is_network = is_network_path(path);
    
    if is_network {
        info!("Using network-optimized playback settings for {}", path);
        // Adjust timeout and buffer settings in playerState if needed
        if let Ok(mut state) = state_arc.lock() {
            state.network_buffering = true;
            state.buffer_progress = 0.2; // Start with some initial progress
        }
    }
    
    // All files now use the same implementation with internal optimizations
    play_audio_file(path, pause_flag, stop_flag, state_arc, playback_position, volume_arc)
}

// Helper function to scan directories with depth limit
pub fn scan_directory_for_audio_files(dir_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    scan_directory_recursively(dir_path, &mut files, 0, MAX_DIRECTORY_DEPTH);
    files
}

// Implementation of recursive directory scanning with depth limit
fn scan_directory_recursively(dir: &Path, files: &mut Vec<PathBuf>, current_depth: usize, max_depth: usize) {
    // Enforce maximum directory depth
    if current_depth > max_depth {
        warn!("Maximum directory scan depth ({}) reached at {:?}", max_depth, dir);
        return;
    }
    
    // Skip hidden directories
    if dir.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("."))
        .unwrap_or(false) 
    {
        debug!("Skipping hidden directory: {:?}", dir);
        return;
    }
    
    info!("Scanning directory at depth {}/{}: {:?}", current_depth, max_depth, dir);
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // Recurse into subdirectory with increased depth
                scan_directory_recursively(&path, files, current_depth + 1, max_depth);
            } else if path.is_file() {
                // Check file extension for supported audio format
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let lowercase_ext = ext.to_lowercase();
                    if get_supported_extensions().iter().any(|supported| &lowercase_ext == supported) {
                        files.push(path);
                    }
                }
            }
        }
    } else {
        warn!("Failed to read directory: {:?}", dir);
    }
}