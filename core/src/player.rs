use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::fs::File;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub status: PlaybackStatus,
    pub current_track: Option<String>,
    pub progress: f32,
    pub volume: f32,
    pub duration: Option<Duration>,
    pub position: Option<Duration>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            current_track: None,
            progress: 0.0,
            volume: 0.8,
            duration: None,
            position: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    Play(String),
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
    Seek(f32),
}

pub struct Player {
    state: Arc<Mutex<PlayerState>>,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    playback_thread: Option<thread::JoinHandle<()>>,
    start_time: Option<Instant>,
    pause_duration: Duration,
}

impl Player {
    pub fn new() -> Self {
        info!("Initializing Media Player...");
        let pause_flag = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::new(AtomicBool::new(false));
        
        Self {
            state: Arc::new(Mutex::new(PlayerState::new())),
            pause_flag,
            stop_flag,
            playback_thread: None,
            start_time: None,
            pause_duration: Duration::ZERO,
        }
    }

    pub fn play(&mut self, path: &str) -> Result<()> {
        info!("Player::play called with path: {}", path);
        
        // Check if file exists before attempting to play
        if !Path::new(path).exists() {
            error!("File does not exist: {}", path);
            return Err(anyhow!("File not found: {}", path));
        }
        
        self.stop();
        self.stop_flag.store(false, Ordering::SeqCst);
        self.pause_flag.store(false, Ordering::SeqCst);
        {
            let mut state = self.state.lock().unwrap();
            state.status = PlaybackStatus::Playing;
            state.current_track = Some(path.to_string());
            state.progress = 0.0;
        }
        self.start_time = Some(Instant::now());
        self.pause_duration = Duration::ZERO;
        let path_string = path.to_string();
        let pause_flag = Arc::clone(&self.pause_flag);
        let stop_flag = Arc::clone(&self.stop_flag);
        let state_arc = Arc::clone(&self.state);
        
        info!("Starting playback thread for path: {}", path);
        self.playback_thread = Some(thread::spawn(move || {
            match play_audio_file(&path_string, pause_flag, stop_flag, state_arc) {
                Ok(_) => info!("Playback finished successfully"),
                Err(e) => error!("Playback error: {}", e),
            }
        }));
        
        info!("Playback thread started successfully");
        Ok(())
    }

    pub fn pause(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            if state.status == PlaybackStatus::Playing {
                info!("Pausing playback");
                self.pause_flag.store(true, Ordering::SeqCst);
                state.status = PlaybackStatus::Paused;
                if let Some(start) = self.start_time {
                    self.pause_duration += start.elapsed();
                    self.start_time = None;
                }
                info!("Playback paused");
            }
        }
    }

    pub fn resume(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            if state.status == PlaybackStatus::Paused {
                info!("Resuming playback");
                self.pause_flag.store(false, Ordering::SeqCst);
                state.status = PlaybackStatus::Playing;
                self.start_time = Some(Instant::now());
                info!("Playback resumed");
            }
        }
    }

    pub fn stop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            if state.status != PlaybackStatus::Stopped {
                info!("Stopping playback");
                self.stop_flag.store(true, Ordering::SeqCst);
                state.status = PlaybackStatus::Stopped;
                state.current_track = None;
                state.progress = 0.0;
                info!("Playback stopped");
                if let Some(handle) = self.playback_thread.take() {
                    let _ = handle.join();
                }
                self.start_time = None;
                self.pause_duration = Duration::ZERO;
            }
        }
    }
    
    pub fn update_progress(&mut self) {
        let should_stop = {
            if let Ok(mut state) = self.state.lock() {
                if state.status == PlaybackStatus::Playing {
                    if let Some(start) = self.start_time {
                        let elapsed = self.pause_duration + start.elapsed();
                        if let Some(duration) = state.duration {
                            let progress = elapsed.as_secs_f32() / duration.as_secs_f32();
                            state.progress = progress.min(1.0);
                            state.position = Some(elapsed);
                            progress >= 1.0
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_stop {
            self.stop();
        }
    }

    pub fn get_state(&self) -> PlayerState {
        self.state.lock().unwrap().clone()
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

// Simple linear interpolation for resampling
fn resample(input: &[f32], input_rate: u32, output_rate: u32, channels: usize) -> Vec<f32> {
    if input_rate == output_rate {
        return input.to_vec();
    }
    
    let ratio = input_rate as f64 / output_rate as f64;
    let input_frames = input.len() / channels;
    let output_frames = (input_frames as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_frames * channels);
    
    for frame in 0..output_frames {
        let src_frame = frame as f64 * ratio;
        let src_frame_i = src_frame as usize;
        let fract = src_frame - src_frame_i as f64;
        
        // Guard against exceeding input bounds
        if src_frame_i >= input_frames - 1 {
            break;
        }
        
        for ch in 0..channels {
            let curr = input[src_frame_i * channels + ch];
            let next = input[(src_frame_i + 1) * channels + ch];
            let sample = curr + fract as f32 * (next - curr);
            output.push(sample);
        }
    }
    
    output
}

fn play_audio_file(
    path: &str, 
    pause_flag: Arc<AtomicBool>, 
    stop_flag: Arc<AtomicBool>,
    state_arc: Arc<Mutex<PlayerState>>
) -> Result<()> {
    info!("Opening file: {}", path);
    
    // Check if file exists
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
    
    info!("Playing: {}", format_name);
    
    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow!("No supported audio track found"))?;
    
    info!("Found audio track: {}", track.codec_params.codec);
    
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|e| anyhow!("Error creating decoder: {}", e))?;
    
    let track_id = track.id;
    let channels = track.codec_params.channels.unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT);
    let channel_count = channels.count();
    let file_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    
    info!("Audio track parameters: {} Hz, {} channels", file_sample_rate, channel_count);
    
    if let Some(n_frames) = track.codec_params.n_frames {
        let duration = Duration::from_secs_f64(n_frames as f64 / file_sample_rate as f64);
        info!("Track duration: {:?}", duration);
        
        if let Ok(mut state) = state_arc.lock() {
            state.duration = Some(duration);
        }
    }
    
    let host = cpal::default_host();
    info!("Using audio host: {}", host.id().name());
    
    let device = host.default_output_device()
        .ok_or_else(|| anyhow!("No output audio device available"))?;
    
    info!("Using output device: {}", device.name().unwrap_or_else(|_| "Unknown".to_string()));
    
    let config_range = device.supported_output_configs()?
        .filter(|c| c.channels() >= channel_count as u16)
        .collect::<Vec<_>>();
    
    if config_range.is_empty() {
        return Err(anyhow!("No suitable output configuration found for device"));
    }
    
    // For FLAC files, which are often 44.1 kHz, try to find a 44.1 kHz output first
    // We're explicitly testing common reference rates to avoid speed issues
    let desired_sample_rates = [44100, 48000, 96000, 192000];
    
    // Try to find an exact match for common sample rates first
    let mut selected_config = None;
    
    for &rate in &desired_sample_rates {
        // Try to match file sample rate first
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
    
    // Warn about sample rate differences
    if file_sample_rate != output_sample_rate {
        info!("Sample rate conversion: {} Hz (file) -> {} Hz (output)", file_sample_rate, output_sample_rate);
    }
    
    // Ensure a reasonable buffer size for audio processing
    let output_buffer_capacity = output_sample_rate as usize * channel_count as usize; // 1 second
    let input_buffer_size = file_sample_rate as usize * channel_count as usize / 4;    // 0.25 seconds
    
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(output_buffer_capacity)));
    let audio_buffer_stream = Arc::clone(&audio_buffer);
    let samples_in_buffer: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let samples_in_buffer_stream = Arc::clone(&samples_in_buffer);
    
    info!("Building audio output stream...");
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut buffer = audio_buffer_stream.lock().unwrap();
            let samples_needed = data.len();
            let samples_available = samples_in_buffer_stream.load(Ordering::Acquire);
            
            if samples_available == 0 {
                // No samples available, output silence
                for sample in data.iter_mut() { *sample = 0.0; }
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
                // Not enough samples, fill the rest with silence
                for sample in &mut data[samples_to_copy..] { *sample = 0.0; }
            }
        },
        |err| { error!("Audio output error: {}", err); },
        None,
    )?;
    
    info!("Starting audio playback stream");
    stream.play()?;
    
    info!("Audio stream started");
    let spec = SignalSpec::new(file_sample_rate, channels);
    let mut sample_buf = SampleBuffer::<f32>::new(4096, spec);
    
    // Temporary buffer for decoded samples that will be sent for resampling
    let mut decode_buffer = Vec::with_capacity(input_buffer_size);
    
    info!("Starting decode loop");
    let mut is_eof = false;
    
    while !is_eof {
        if stop_flag.load(Ordering::SeqCst) {
            info!("Stop signal received; exiting decode loop");
            break;
        }
        
        if pause_flag.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(20));
            continue;
        }
        
        // Only decode more if we have room in the buffer
        let current_samples = samples_in_buffer.load(Ordering::Acquire);
        if current_samples >= output_buffer_capacity / 2 {
            // Buffer is half full - wait a bit to avoid overflows
            thread::sleep(Duration::from_millis(5));
            continue;
        }
        
        // Decode a batch of packets to fill our decode buffer
        let batch_size = (input_buffer_size / 4096) + 1; // How many packets to read before processing
        let mut packets_read = 0;
        
        for _ in 0..batch_size {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            
            let packet = match format.next_packet() {
                Ok(packet) => { 
                    if packet.track_id() != track_id { 
                        continue; 
                    } 
                    packet 
                },
                Err(symphonia::core::errors::Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    info!("End of stream reached");
                    is_eof = true;
                    break;
                },
                Err(e) => { 
                    warn!("Error reading packet: {}", e); 
                    continue; 
                }
            };
            
            packets_read += 1;
            
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Get interleaved samples
                    sample_buf.copy_interleaved_ref(decoded);
                    let samples = sample_buf.samples();
                    
                    // Accumulate samples into the decode buffer
                    decode_buffer.extend_from_slice(samples);
                },
                Err(e) => { 
                    warn!("Error decoding packet: {}", e); 
                }
            }
        }
        
        // Process the samples if we've read anything or reached the end
        if !decode_buffer.is_empty() && (packets_read > 0 || is_eof) {
            // Apply resampling if needed
            let output_samples = if file_sample_rate != output_sample_rate {
                resample(&decode_buffer, file_sample_rate, output_sample_rate, channel_count)
            } else {
                decode_buffer.clone()
            };
            
            // Lock the buffer and add the processed samples
            let mut buffer = audio_buffer.lock().unwrap();
            let current_len = buffer.len();
            let samples_to_add = output_samples.len();
            
            if current_len + samples_to_add <= output_buffer_capacity {
                buffer.extend_from_slice(&output_samples);
                samples_in_buffer.store(current_len + samples_to_add, Ordering::Release);
            } else {
                warn!("Buffer overflow prevented - dropping {} samples", samples_to_add);
            }
            
            // Clear the decode buffer for the next batch
            decode_buffer.clear();
        }
    }
    
    info!("Decoding complete");
    
    // Wait for buffer to empty before returning
    let max_wait = Duration::from_secs(5);
    let wait_start = Instant::now();
    
    while samples_in_buffer.load(Ordering::Acquire) > 0 && wait_start.elapsed() < max_wait {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    
    info!("Playback finished");
    Ok(())
}