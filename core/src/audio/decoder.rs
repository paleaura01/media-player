// core/src/audio/decoder.rs
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
use ffmpeg_sys_next as ffmpeg;
// Add these imports for the FFmpeg enum variants
use ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_AUDIO;
use ffmpeg_sys_next::AVSampleFormat::AV_SAMPLE_FMT_FLT;
use ffmpeg_sys_next::AVRounding::AV_ROUND_UP;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use crate::audio::buffer::AudioRingBuffer;
use crate::audio::position::PlaybackPosition;
use crate::PlayerState;

// Initialize FFmpeg only once
static mut FFMPEG_INITIALIZED: bool = false;

// Helper function to convert C string to Rust string
unsafe fn to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

// Helper to get a const pointer to a slice
#[allow(dead_code)]
fn as_ptr<T>(slice: &[T]) -> *const T {
    slice.as_ptr()
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
    // Initialize FFmpeg if needed
    if let Err(e) = initialize_ffmpeg() {
        error!("Failed to initialize FFmpeg: {}", e);
        return vec!["mp3".to_string(), "wav".to_string()]; // Fallback
    }
    
    // Common audio formats as fallback
    let common_extensions = [
        "mp3", "wav", "flac", "ogg", "m4a", "aac", "opus", 
        "wma", "ape", "mka", "mp4", "mp2", "ac3", "amr", "au",
        "mid", "midi", "ra", "rm", "tta", "wv", "caf", "aiff"
    ];
    
    common_extensions.iter().map(|&s| s.to_string()).collect()
}

// Check if a file is supported by FFmpeg
pub fn is_supported_audio_format(path: &str) -> bool {
    // Initialize FFmpeg if needed
    if let Err(_) = initialize_ffmpeg() {
        return false;
    }
    
    // For network paths, assume supported based on extension
    if path.starts_with("\\\\") || path.contains("://") {
        let lowercase_path = path.to_lowercase();
        return get_supported_extensions().iter().any(|ext| lowercase_path.ends_with(&format!(".{}", ext)));
    }
    
    // Check if local file exists
    if !Path::new(path).exists() {
        return false;
    }
    
    // Try to open the file with FFmpeg
    unsafe {
        let c_path = match CString::new(path) {
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
            
            // Use the imported enum variant
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

// Safe audio frame processing for minimal memory usage during prebuffering
unsafe fn process_audio_frame_minimal(
    frame: *mut ffmpeg::AVFrame,
    swr_ctx: *mut ffmpeg::SwrContext,
    channel_count: usize,
    output_sample_rate: u32,
    file_sample_rate: u32,
    volume_arc: Arc<Mutex<f32>>,
    ring_buffer: Arc<Mutex<AudioRingBuffer>>
) {
    if frame.is_null() {
        return;
    }
    
    let nb_samples = (*frame).nb_samples;
    if nb_samples <= 0 || nb_samples > 8192 {
        // Skip too large frames during prebuffering
        return;
    }
    
    // Use small fixed buffer sizes
    let input_samples = nb_samples as i32;
    let output_samples = std::cmp::min(
        ffmpeg::av_rescale_rnd(
            input_samples as i64 * output_sample_rate as i64,
            file_sample_rate as i64,
            file_sample_rate as i64,
            AV_ROUND_UP
        ) as i32,
        8192 // Very limited during prebuffering
    );
    
    // Allocate output buffer
    let mut output_channels_ptr: *mut *mut u8 = std::ptr::null_mut();
    let alloc_result = ffmpeg::av_samples_alloc_array_and_samples(
        &mut output_channels_ptr,
        std::ptr::null_mut(),
        channel_count as i32,
        output_samples,
        AV_SAMPLE_FMT_FLT,
        0
    );
    
    if alloc_result < 0 || output_channels_ptr.is_null() {
        return;
    }
    
    // Convert audio
    let out_samples = ffmpeg::swr_convert(
        swr_ctx,
        output_channels_ptr,
        output_samples,
        (*frame).extended_data as *mut *const u8,
        input_samples
    );
    
    if out_samples <= 0 || out_samples > output_samples {
        ffmpeg::av_freep(output_channels_ptr as *mut libc::c_void);
        return;
    }
    
    // Process a manageable chunk of data
    let max_process = std::cmp::min(out_samples as usize, 4096);
    let output_channels = output_channels_ptr as *mut *mut f32;
    
    // Use stack-allocated fixed-size buffer
    let mut temp_buffer = [0.0f32; 8192]; // Will be on stack but small size
    let buffer_len = std::cmp::min(max_process * channel_count, temp_buffer.len());
    
    // Fill the temporary buffer with safety checks
    let mut pos = 0;
    for i in 0..max_process {
        for ch in 0..channel_count {
            if pos < buffer_len && ch < channel_count && i < max_process {
                if !(*output_channels.offset(ch as isize)).is_null() {
                    temp_buffer[pos] = *(*output_channels.offset(ch as isize)).offset(i as isize);
                }
                pos += 1;
            }
        }
    }
    
    // Apply volume
    let volume = match volume_arc.lock() {
        Ok(v) => *v,
        Err(_) => 1.0,
    };
    
    for i in 0..buffer_len {
        temp_buffer[i] *= volume;
    }
    
    // Write to ring buffer
    if let Ok(mut rb) = ring_buffer.lock() {
        rb.write_safe(&temp_buffer[0..buffer_len]);
    }
    
    // Free allocated memory
    ffmpeg::av_freep(output_channels_ptr as *mut libc::c_void);
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
    if nb_samples <= 0 || nb_samples > 16384 {
        warn!("Skipping abnormally sized frame: {} samples", nb_samples);
        return 0;
    }
    
    // Using smaller batches for network files
    const MAX_BATCH_SIZE: usize = 4096;
    
    // Calculate how many samples we need in output
    let input_samples = nb_samples as i32;
    let output_samples = std::cmp::min(
        ffmpeg::av_rescale_rnd(
            input_samples as i64 * output_sample_rate as i64,
            file_sample_rate as i64,
            file_sample_rate as i64,
            AV_ROUND_UP
        ) as i32,
        MAX_BATCH_SIZE as i32
    );
    
    // Allocate output buffer
    let mut output_channels_ptr: *mut *mut u8 = std::ptr::null_mut();
    let alloc_result = ffmpeg::av_samples_alloc_array_and_samples(
        &mut output_channels_ptr,
        std::ptr::null_mut(),
        channel_count as i32,
        output_samples,
        AV_SAMPLE_FMT_FLT,
        0
    );
    
    if alloc_result < 0 || output_channels_ptr.is_null() {
        warn!("Failed to allocate output buffer: {}", alloc_result);
        return 0;
    }
    
    // Convert audio with smaller buffer
    let out_samples = ffmpeg::swr_convert(
        swr_ctx,
        output_channels_ptr,
        output_samples,
        (*frame).extended_data as *mut *const u8,
        input_samples
    );
    
    if out_samples <= 0 || out_samples > output_samples {
        warn!("Error resampling audio: {}", out_samples);
        ffmpeg::av_freep(output_channels_ptr as *mut libc::c_void);
        return 0;
    }
    
    // Use fixed-size stack buffer for safety
    let mut stack_buffer = [0.0f32; MAX_BATCH_SIZE * 8]; // 8 is max channels we support
    let total_samples = out_samples as usize * channel_count;
    let safe_samples = std::cmp::min(total_samples, stack_buffer.len());
    
    // Cast to the right pointer type
    let output_channels = output_channels_ptr as *mut *mut f32;
    
    // Fill interleaved buffer with extra safety checks
    let mut written = 0;
    for i in 0..out_samples as usize {
        for ch in 0..channel_count {
            if written < safe_samples {
                if ch < channel_count {
                    let channel_ptr = *output_channels.offset(ch as isize);
                    if !channel_ptr.is_null() && i < out_samples as usize {
                        stack_buffer[written] = *channel_ptr.offset(i as isize);
                    } else {
                        stack_buffer[written] = 0.0;
                    }
                } else {
                    stack_buffer[written] = 0.0;
                }
                written += 1;
            }
        }
    }
    
    // Apply volume
    let volume = match volume_arc.lock() {
        Ok(v) => *v,
        Err(_) => 1.0,
    };
    
    for i in 0..written {
        stack_buffer[i] *= volume;
    }
    
    // Write to ring buffer
    let buffer_health = {
        if let Ok(mut rb) = ring_buffer.lock() {
            let _written = rb.write_safe(&stack_buffer[0..written]);
            
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
    
    // Free allocated memory
    ffmpeg::av_freep(output_channels_ptr as *mut libc::c_void);
    
    // Reset needs_data flag if we've processed enough data
    if buffer_health > 0.5 {
        needs_data.store(false, Ordering::Release);
    }
    
    // Update last buffer warning time
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
    // Use the enhanced version with default options
    play_audio_file_with_options(
        path,
        pause_flag,
        stop_flag,
        state_arc,
        playback_position,
        volume_arc,
        None,  // default buffer size
        false  // not network mode
    )
}

// Enhanced version that accepts additional parameters for network playback
pub fn play_audio_file_enhanced(
    path: &str,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    state_arc: Arc<Mutex<PlayerState>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume_arc: Arc<Mutex<f32>>,
    prefetch_mode: bool,  // Enable prefetch mode for network files
    buffer_size: Option<usize>,  // Optional custom buffer size
) -> Result<()> {
    // Log the enhanced playback settings
    if prefetch_mode {
        info!("Starting enhanced playback with prefetch mode enabled");
        
        // Check if this is a network path
        let is_network = path.starts_with("\\\\") || path.contains("://");
        if is_network {
            info!("Network path detected, using enhanced buffering");
            
            // Update buffer health in the playback position
            if let Ok(mut pos) = playback_position.lock() {
                pos.buffer_health = Some(0.0); // Will be updated during playback
            }
            
            // Update player state for UI
            if let Ok(mut st) = state_arc.lock() {
                st.network_buffering = true;
                st.buffer_progress = 0.0;
            }
        }
    }
    
    // For network paths, calculate appropriate buffer size with safety limits
    let is_network = path.starts_with("\\\\") || path.contains("://");
    let custom_buffer_size = if is_network {
        // For network paths, determine format-specific buffer size
        let path_lower = path.to_lowercase();
        if path_lower.ends_with(".flac") {
            // FLAC files are uncompressed and need larger buffers, but with safety limits
            Some(buffer_size.unwrap_or(4 * 1024 * 1024)) // 4MB for FLAC
        } else if path_lower.ends_with(".wav") {
            // WAV files are also uncompressed
            Some(buffer_size.unwrap_or(4 * 1024 * 1024)) // 4MB for WAV
        } else {
            // For compressed formats like MP3
            Some(buffer_size.unwrap_or(2 * 1024 * 1024)) // 2MB for other formats
        }
    } else {
        buffer_size
    };
    
    // Call the core playback function with enhanced options
    play_audio_file_with_options(
        path, 
        pause_flag, 
        stop_flag, 
        state_arc, 
        playback_position, 
        volume_arc,
        custom_buffer_size,
        is_network
    )
}

// Core playback function with additional options
fn play_audio_file_with_options(
    path: &str,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    state_arc: Arc<Mutex<PlayerState>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume_arc: Arc<Mutex<f32>>,
    custom_buffer_size: Option<usize>,
    is_network: bool,
) -> Result<()> {
    info!("Opening file with FFmpeg: {}", path);

    // Initialize FFmpeg if needed
    initialize_ffmpeg()?;

    // For local files, check if the file exists
    if !is_network && !Path::new(path).exists() {
        error!("File does not exist: {}", path);
        return Err(anyhow!("File does not exist: {}", path));
    }

    // Network file specific logging
    if is_network {
        info!("Using enhanced buffer settings for network file");
        if let Some(size) = custom_buffer_size {
            info!("Network buffer size: {}KB", size / 1024);
        }
    }

    unsafe {
        // Try to create a C string from the path
        let c_path = match CString::new(path) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid path string: {}", e);
                return Err(anyhow!("Invalid path string: {}", e));
            }
        };
        
        // Create format context with proper error handling
        let mut format_ctx: *mut ffmpeg::AVFormatContext = std::ptr::null_mut();
        
        // For network files, create format options with longer timeouts
        let mut options: *mut ffmpeg::AVDictionary = std::ptr::null_mut();
        if is_network {
            // Set timeout values for network operations
            let timeout_key = CString::new("timeout").unwrap();
            let rw_timeout_key = CString::new("rw_timeout").unwrap();
            let timeout_val = CString::new("10000000").unwrap(); // 10 seconds in microseconds
            
            ffmpeg::av_dict_set(&mut options, timeout_key.as_ptr(), timeout_val.as_ptr(), 0);
            ffmpeg::av_dict_set(&mut options, rw_timeout_key.as_ptr(), timeout_val.as_ptr(), 0);
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
            let err_buf = [0i8; 1024];
            ffmpeg::av_strerror(ret, err_buf.as_ptr() as *mut i8, 1024);
            let error_msg = to_string(err_buf.as_ptr());
            error!("Could not open input file: {} ({})", error_msg, ret);
            return Err(anyhow!("Could not open input file: {}", error_msg));
        }
        
        // Find stream info with better error handling
        let ret = ffmpeg::avformat_find_stream_info(format_ctx, std::ptr::null_mut());
        if ret < 0 {
            let err_buf = [0i8; 1024];
            ffmpeg::av_strerror(ret, err_buf.as_ptr() as *mut i8, 1024);
            let error_msg = to_string(err_buf.as_ptr());
            ffmpeg::avformat_close_input(&mut format_ctx);
            error!("Could not find stream information: {} ({})", error_msg, ret);
            return Err(anyhow!("Could not find stream information: {}", error_msg));
        }
        
        // Find best audio stream
        let mut audio_stream_idx: i32 = -1;
        for i in 0..(*format_ctx).nb_streams {
            let stream = *(*format_ctx).streams.offset(i as isize);
            if (*(*stream).codecpar).codec_type == AVMEDIA_TYPE_AUDIO {
                audio_stream_idx = i as i32;
                break;
            }
        }
        
        if audio_stream_idx == -1 {
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not find any audio stream in the file"));
        }
        
        // Get stream
        let stream = *(*format_ctx).streams.offset(audio_stream_idx as isize);
        let codec_params = (*stream).codecpar;
        
        // Find decoder
        let codec = ffmpeg::avcodec_find_decoder((*codec_params).codec_id);
        if codec.is_null() {
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Unsupported codec"));
        }
        
        // Create codec context
        let codec_ctx = ffmpeg::avcodec_alloc_context3(codec);
        if codec_ctx.is_null() {
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not allocate codec context"));
        }
        
        // Copy codec params to codec context
        if ffmpeg::avcodec_parameters_to_context(codec_ctx, codec_params) < 0 {
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not copy codec parameters to context"));
        }
        
        // Open codec
        if ffmpeg::avcodec_open2(codec_ctx, codec, std::ptr::null_mut()) < 0 {
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Could not open codec"));
        }
        
        // Get audio parameters - Using ch_layout
        let channel_count = (*codec_ctx).ch_layout.nb_channels as usize;
        let file_sample_rate = (*codec_ctx).sample_rate as u32;
        
        info!("Found audio track: codec={}, {} ch, {} Hz",
              to_string((*codec).name), channel_count, file_sample_rate);
        
        // Set up audio output with cpal
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
        info!("Output config: {} ch, {} Hz", config.channels, output_sample_rate);
    
        // Calculate total duration for progress tracking
        let duration_seconds = if (*stream).duration > 0 {
            // Convert time_base to seconds
            let time_base = (*stream).time_base;
            (*stream).duration as f64 * time_base.num as f64 / time_base.den as f64
        } else if (*format_ctx).duration > 0 {
            // Use container duration as fallback
            (*format_ctx).duration as f64 / ffmpeg::AV_TIME_BASE as f64
        } else {
            // Default duration if unknown
            300.0
        };
        
        let track_duration = Duration::from_secs_f64(duration_seconds);
        info!("Track duration: {:?}", track_duration);
        
        // Calculate total samples based on duration
        let total_samples = (duration_seconds * file_sample_rate as f64) as u64 * channel_count as u64;
        
        {
            let mut pos = playback_position.lock().unwrap();
            pos.set_total_samples(total_samples);
            pos.set_channel_count(channel_count);
            pos.sample_rate = file_sample_rate;
            pos.buffer_health = Some(0.0); // Initialize buffer health
        }
    
        if let Ok(mut st) = state_arc.lock() {
            st.duration = Some(track_duration);
            if is_network {
                st.network_buffering = true;
                st.buffer_progress = 0.0;
            }
        }
        
        // Calculate buffer size based on format and network status with safety limits
        let buffer_size_frames = match custom_buffer_size {
            Some(size) => {
                // Cap maximum buffer size to prevent overflow
                let max_safe_size = 16 * 1024 * 1024; // 16MB max
                let limited_size = std::cmp::min(size, max_safe_size);
                limited_size / (channel_count * std::mem::size_of::<f32>())
            },
            None => {
                if is_network {
                    // For network files, use a safer buffer size
                    std::cmp::min(
                        output_sample_rate as usize * channel_count,
                        // Cap at 2MB for safety
                        2 * 1024 * 1024 / (channel_count * std::mem::size_of::<f32>())
                    )
                } else {
                    // Standard buffer for local files
                    (output_sample_rate as usize * channel_count) / 5
                }
            }
        };
        
        // Set up ring buffer for audio output - larger for network paths but with safety limits
        let ring_buffer_size = if is_network {
            // Limit multiplication for network paths
            std::cmp::min(buffer_size_frames * 4, 
                          8 * 1024 * 1024 / (channel_count * std::mem::size_of::<f32>()))
        } else {
            buffer_size_frames * 2
        };
        
        let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(ring_buffer_size)));
        let ring_buffer_stream = Arc::clone(&ring_buffer);
        
        let needs_data = Arc::new(AtomicBool::new(true));
        let needs_data_stream = Arc::clone(&needs_data);
    
        // Set up audio output stream
        let audio_stream = device.build_output_stream(
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
    
        audio_stream.play().map_err(|e| anyhow!("Failed to start audio: {}", e))?;
        
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
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to allocate resampler context"));
        }
        
        // Set options on the SwrContext
        ffmpeg::av_opt_set_int(swr_ctx as *mut _, 
                               CString::new("in_sample_rate")?.as_ptr(), 
                               file_sample_rate as i64, 0);
        ffmpeg::av_opt_set_int(swr_ctx as *mut _, 
                               CString::new("out_sample_rate")?.as_ptr(), 
                               output_sample_rate as i64, 0);
        ffmpeg::av_opt_set_sample_fmt(swr_ctx as *mut _, 
                                      CString::new("in_sample_fmt")?.as_ptr(), 
                                      (*codec_ctx).sample_fmt, 0);
        ffmpeg::av_opt_set_sample_fmt(swr_ctx as *mut _, 
                                      CString::new("out_sample_fmt")?.as_ptr(), 
                                      AV_SAMPLE_FMT_FLT, 0);  // Use the imported enum variant
        
        // Set channel layouts
        ffmpeg::av_opt_set_chlayout(swr_ctx as *mut _,
                                   CString::new("in_chlayout")?.as_ptr(),
                                   &in_ch_layout,
                                   0);
                                   
        ffmpeg::av_opt_set_chlayout(swr_ctx as *mut _,
                                   CString::new("out_chlayout")?.as_ptr(),
                                   &out_ch_layout,
                                   0);
        
        // Initialize the resampler
        if ffmpeg::swr_init(swr_ctx) < 0 {
            ffmpeg::swr_free(&mut swr_ctx);
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to initialize resampler"));
        }
        
        // Allocate packet and frame
        let packet = ffmpeg::av_packet_alloc();
        if packet.is_null() {
            ffmpeg::swr_free(&mut swr_ctx);
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to allocate packet"));
        }
        
        let frame = ffmpeg::av_frame_alloc();
        if frame.is_null() {
            ffmpeg::av_packet_free(&mut (packet as *mut _));
            ffmpeg::swr_free(&mut swr_ctx);
            ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
            ffmpeg::avformat_close_input(&mut format_ctx);
            return Err(anyhow!("Failed to allocate frame"));
        }
        
        // Pre-buffering for network files with improved safety
        if is_network {
            // Use much smaller buffer for safety
            let safe_buffer_size = std::cmp::min(buffer_size_frames / 4, 32768);
            let pre_buffer_timeout = Instant::now() + Duration::from_secs(10);
            
            info!("Pre-buffering network audio - waiting for {}KB of data", 
                 safe_buffer_size * channel_count * std::mem::size_of::<f32>() / 1024);
            
            // Set buffer_progress on PlayerState
            let set_buffer_progress = |progress: f32| {
                if let Ok(mut st) = state_arc.lock() {
                    st.buffer_progress = progress;
                }
            };
            
            // Prebuffering phase with aggressive safety limits
            let mut buffer_packets = true;
            let mut last_warn_time = Instant::now();
            let mut packet_count = 0;
            let max_packets = 100; // Very conservative limit
            
            // Skip prebuffering completely for very long tracks
            if duration_seconds > 3600.0 {
                info!("Long track detected ({}s), skipping prebuffering", duration_seconds);
                if let Ok(mut st) = state_arc.lock() {
                    st.network_buffering = false;
                    st.buffer_progress = 1.0;
                }
            } else {
                while buffer_packets && 
                      Instant::now() < pre_buffer_timeout && 
                      !stop_flag.load(Ordering::SeqCst) &&
                      packet_count < max_packets {
                      
                    packet_count += 1;
                    
                    // Process only a few packets to avoid memory issues
                    if let Ok(rb) = ring_buffer.lock() {
                        let available = rb.available();
                        let health = (available as f32) / (safe_buffer_size as f32);
                        
                        // Only start with minimal buffering
                        if health >= 0.2 {
                            info!("Basic pre-buffering complete ({:.1}%), starting playback", health * 100.0);
                            if let Ok(mut st) = state_arc.lock() {
                                st.network_buffering = false;
                                st.buffer_progress = health;
                            }
                            break;
                        }
                        
                        set_buffer_progress(health.min(1.0));
                        
                        if Instant::now() - last_warn_time > Duration::from_millis(500) {
                            info!("Network buffer pre-loading: {:.1}%", health * 100.0);
                            last_warn_time = Instant::now();
                        }
                    }
                    
                    // Process just one packet at a time, minimizing memory usage
                    let ret = ffmpeg::av_read_frame(format_ctx, packet);
                    if ret < 0 {
                        if ret == ffmpeg::AVERROR_EOF || ret == ffmpeg::AVERROR(ffmpeg::EAGAIN) {
                            info!("Reached EOF during pre-buffering");
                            buffer_packets = false;
                        } else {
                            warn!("Error reading frame during pre-buffering: {}", ret);
                        }
                        continue;
                    }
                    
                    if (*packet).stream_index != audio_stream_idx {
                        ffmpeg::av_packet_unref(packet);
                        continue;
                    }
                    
                    let ret = ffmpeg::avcodec_send_packet(codec_ctx, packet);
                    ffmpeg::av_packet_unref(packet);
                    
                    if ret < 0 {
                        warn!("Error sending packet to decoder during pre-buffering: {}", ret);
                        continue;
                    }
                    
                    // Process a single frame then continue the loop
                    let ret = ffmpeg::avcodec_receive_frame(codec_ctx, frame);
                    if ret < 0 && ret != ffmpeg::AVERROR(ffmpeg::EAGAIN) && ret != ffmpeg::AVERROR_EOF {
                        warn!("Error receiving frame: {}", ret);
                        continue;
                    }
                    
                    if ret >= 0 {
                        // Just a minimal processing of the frame - avoiding stack issues
                        process_audio_frame_minimal(
                            frame, 
                            swr_ctx,
                            channel_count,
                            output_sample_rate,
                            file_sample_rate,
                            volume_arc.clone(),
                            ring_buffer.clone()
                        );
                        
                        ffmpeg::av_frame_unref(frame);
                    }
                    
                    // Short sleep to avoid CPU spinning
                    thread::sleep(Duration::from_millis(1));
                }
            }
            
            // Continue playback regardless of buffer state
            info!("Starting playback");
            if let Ok(mut st) = state_arc.lock() {
                st.network_buffering = false;
                st.buffer_progress = 1.0;
            }
        }
        
        info!("Starting FFmpeg decode loop for track: {}", path);
        let mut current_frames: u64 = 0;
        let mut is_eof = false;
        let mut last_debug_log = Instant::now();
        let mut last_buffer_warn = Instant::now();
        
        // Main decoding loop with safety improvements
        while !is_eof && !stop_flag.load(Ordering::SeqCst) {
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
                info!("Seek requested -> {:.4} ({:.2}%)", target_fraction, target_fraction * 100.0);
                
                // Clear the ring buffer first
                if let Ok(mut rb) = ring_buffer.lock() {
                    *rb = AudioRingBuffer::new(rb.capacity());
                }
                
                // Calculate seek position in seconds and convert to stream timebase
                let target_time_seconds = target_fraction * duration_seconds as f32;
                let timestamp = (target_time_seconds * (*stream).time_base.den as f32 / 
                                (*stream).time_base.num as f32) as i64;
                
                info!("Seeking to {:.2}s (timestamp: {})", target_time_seconds, timestamp);
                
                // Flush buffers
                ffmpeg::avcodec_flush_buffers(codec_ctx);
                
                // Perform FFmpeg seek
                let seek_flags = ffmpeg::AVSEEK_FLAG_BACKWARD;
                let ret = ffmpeg::av_seek_frame(
                    format_ctx, 
                    audio_stream_idx,
                    timestamp,
                    seek_flags
                );
                
                if ret >= 0 {
                    info!("Seek succeeded");
                    
                    // Update playback position
                    let new_frames = (target_fraction * total_samples as f32 / channel_count as f32) as u64;
                    current_frames = new_frames * channel_count as u64;
                    
                    if let Ok(pos) = playback_position.lock() {
                        let frame_pos = (current_frames / channel_count as u64) as usize;
                        pos.set_current_frame(frame_pos);
                        
                        let frame_count = total_samples / channel_count as u64;
                        let progress = if frame_count > 0 {
                            frame_pos as f64 / frame_count as f64
                        } else {
                            0.0
                        };
                        
                        info!("Updated frame position to {} of {} ({:.4}%)",
                             frame_pos, frame_count, progress * 100.0);
                    }
                    
                    // For network files, wait for some buffering after seek
                    if is_network {
                        if let Ok(mut st) = state_arc.lock() {
                            st.network_buffering = true;
                            st.buffer_progress = 0.0;
                        }
                        
                        let mut last_post_seek_log = Instant::now();
                        let post_seek_timeout = Instant::now() + Duration::from_secs(5);
                        let buffer_threshold = buffer_size_frames / 10; // Need less buffer after seek
                        
                        while Instant::now() < post_seek_timeout && !stop_flag.load(Ordering::SeqCst) {
                            let buffer_health = {
                                if let Ok(rb) = ring_buffer.lock() {
                                    let available = rb.available();
                                    let health = (available as f32) / (buffer_threshold as f32);
                                    
                                    if let Ok(mut st) = state_arc.lock() {
                                        st.buffer_progress = health.min(1.0);
                                    }
                                    
                                    health
                                } else {
                                    0.0
                                }
                            };
                            
                            if Instant::now() - last_post_seek_log > Duration::from_millis(200) {
                                info!("Post-seek buffering: {:.1}%", buffer_health * 100.0);
                                last_post_seek_log = Instant::now();
                            }
                            
                            if buffer_health >= 1.0 {
                                break;
                            }
                            
                            thread::sleep(Duration::from_millis(20));
                        }
                        
                        if let Ok(mut st) = state_arc.lock() {
                            st.network_buffering = false;
                            st.buffer_progress = 1.0;
                        }
                    }
                } else {
                    warn!("Seeking failed with error: {}", ret);
                }
                
                needs_data.store(true, Ordering::Release);
                continue;
            }
    
            if !needs_data.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            
            // Read packet
            let ret = ffmpeg::av_read_frame(format_ctx, packet);
            if ret < 0 {
                // End of file or error
                if ret == ffmpeg::AVERROR_EOF || ret == ffmpeg::AVERROR(ffmpeg::EAGAIN) {
                    info!("End of file reached");
                    is_eof = true;
                    
                    // Track was played completely, increment play count
                    if let Ok(mut st) = state_arc.lock() {
                        st.track_completed = true;
                    }
                } else {
                    warn!("Error reading frame: {}", ret);
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
                warn!("Error sending packet to decoder: {}", ret);
                continue;
            }
            
            // Process frames with conservative approach
            let mut frame_count = 0;
            const MAX_FRAMES_PER_PACKET: usize = 16;
            
            while frame_count < MAX_FRAMES_PER_PACKET {
                frame_count += 1;
                
                let ret = ffmpeg::avcodec_receive_frame(codec_ctx, frame);
                if ret == ffmpeg::AVERROR(ffmpeg::EAGAIN) || ret == ffmpeg::AVERROR_EOF {
                    break;
                } else if ret < 0 {
                    warn!("Error receiving frame from decoder: {}", ret);
                    break;
                }
                
                // For network files, use a more conservative approach
                let frames_decoded = if is_network {
                    process_audio_frame_safe(
                        frame,
                        swr_ctx, 
                        channel_count,
                        output_sample_rate,
                        file_sample_rate,
                        volume_arc.clone(),
                        ring_buffer.clone(),
                        &needs_data,
                        &mut last_buffer_warn
                    )
                } else {
                    // For local files, use standard safe processing
                    process_audio_frame_safe(
                        frame,
                        swr_ctx, 
                        channel_count,
                        output_sample_rate,
                        file_sample_rate,
                        volume_arc.clone(),
                        ring_buffer.clone(),
                        &needs_data,
                        &mut last_buffer_warn
                    )
                };
                
                // Update position
                current_frames += frames_decoded * channel_count as u64;
                if let Ok(pos) = playback_position.lock() {
                    pos.update_current_sample(frames_decoded as usize);
                    // Buffer health already updated in process_audio_frame_safe
                }
                
                // For network files, process fewer frames per packet iteration
                if is_network && frame_count >= 4 {
                    break;
                }
            }
            
            if last_debug_log.elapsed() >= Duration::from_millis(500) {
                let cur_seconds = current_frames as f64 / (file_sample_rate.max(1) as f64 * channel_count as f64);
                debug!(
                    "Current position: {:.3}s / {:.3}s ({:.2}%)",
                    cur_seconds,
                    duration_seconds,
                    (cur_seconds / duration_seconds) * 100.0
                );
                last_debug_log = Instant::now();
            }
        }
        
        // Clean up
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
        
        // Free resources
        ffmpeg::av_frame_free(&mut (frame as *mut _));
        ffmpeg::av_packet_free(&mut (packet as *mut _));
        ffmpeg::swr_free(&mut swr_ctx);
        ffmpeg::avcodec_free_context(&mut (codec_ctx as *mut _));
        ffmpeg::avformat_close_input(&mut format_ctx);
        
        info!("Playback finished.");
    }
    
    Ok(())
}