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

// Added for accurate timing
struct PlaybackPosition {
    total_samples: u64,
    current_sample: Arc<AtomicUsize>,
    sample_rate: u32,
}

impl PlaybackPosition {
    fn new(sample_rate: u32) -> Self {
        Self {
            total_samples: 0,
            current_sample: Arc::new(AtomicUsize::new(0)),
            sample_rate,
        }
    }

    fn set_total_samples(&mut self, total_samples: u64) {
        self.total_samples = total_samples;
    }

    // Fixed to prevent overflow by using saturating_add
    fn update_current_sample(&self, samples: usize) {
        let current = self.current_sample.load(Ordering::Relaxed);
        let new_value = current.saturating_add(samples);
        self.current_sample.store(new_value, Ordering::Relaxed);
    }

    fn reset(&self) {
        self.current_sample.store(0, Ordering::Relaxed);
    }

    fn progress(&self) -> f32 {
        if self.total_samples == 0 {
            return 0.0;
        }
        let current = self.current_sample.load(Ordering::Relaxed) as f64;
        let total = self.total_samples as f64;
        (current / total).min(1.0) as f32
    }

    fn position(&self) -> Duration {
        let current = self.current_sample.load(Ordering::Relaxed);
        // Prevent division by zero
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = current as f64 / (sample_rate * 2.0); // Stereo
        Duration::from_secs_f64(seconds)
    }

    fn duration(&self) -> Duration {
        // Prevent division by zero
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = self.total_samples as f64 / (sample_rate * 2.0); // Stereo
        Duration::from_secs_f64(seconds)
    }

    fn seek(&self, progress: f32) {
        if self.total_samples == 0 {
            return;
        }
        
        // Clamp progress to valid range
        let progress = progress.max(0.0).min(1.0);
        let new_position = (progress as f64 * self.total_samples as f64) as usize;
        self.current_sample.store(new_position, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub status: PlaybackStatus,
    pub current_track: Option<String>,
    pub progress: f32,
    pub volume: f32,
    pub duration: Option<Duration>,
    pub position: Option<Duration>,
    pub shuffle_enabled: bool,  // Added shuffle_enabled field
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
            shuffle_enabled: false,  // Initialize to false
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
    Shuffle,      // Added for shuffle functionality
    NextTrack,    // Added for next track
    PreviousTrack, // Added for previous track
}

pub struct Player {
    pub state: Arc<Mutex<PlayerState>>,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    playback_thread: Option<thread::JoinHandle<()>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume: Arc<Mutex<f32>>,
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
            playback_position: Arc::new(Mutex::new(PlaybackPosition::new(44100))),
            volume: Arc::new(Mutex::new(0.8)),
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
        
        // Reset playback position
        {
            let position = self.playback_position.lock().unwrap();
            position.reset();
        }
        
        {
            let mut state = self.state.lock().unwrap();
            state.status = PlaybackStatus::Playing;
            state.current_track = Some(path.to_string());
            state.progress = 0.0;
        }
        
        let path_string = path.to_string();
        let pause_flag = Arc::clone(&self.pause_flag);
        let stop_flag = Arc::clone(&self.stop_flag);
        let state_arc = Arc::clone(&self.state);
        let playback_position = Arc::clone(&self.playback_position);
        let volume = Arc::clone(&self.volume);
        
        info!("Starting playback thread for path: {}", path);
        self.playback_thread = Some(thread::spawn(move || {
            match play_audio_file(&path_string, pause_flag, stop_flag, state_arc, playback_position, volume) {
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
                
                // Wait a tiny bit to make sure the audio thread acknowledges the pause
                // This makes the pause response feel more immediate
                thread::sleep(Duration::from_millis(5));
                
                state.status = PlaybackStatus::Paused;
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
                info!("Playback resumed");
            }
        }
    }

    pub fn stop(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            if state.status != PlaybackStatus::Stopped {
                info!("Stopping playback");
                self.stop_flag.store(true, Ordering::SeqCst);
                
                // Clear audio buffer and immediately reset status
                thread::sleep(Duration::from_millis(10));
                
                state.status = PlaybackStatus::Stopped;
                state.current_track = None;
                state.progress = 0.0;
                state.position = None;
                state.duration = None;
                
                info!("Playback stopped");
                
                // Reset playback position
                {
                    let position = self.playback_position.lock().unwrap();
                    position.reset();
                }
                
                if let Some(handle) = self.playback_thread.take() {
                    let _ = handle.join();
                }
            }
        }
    }
    
    pub fn update_progress(&mut self) {
        // Added try-catch style handling for the mutex locks
        let progress_info = match self.playback_position.try_lock() {
            Ok(position_guard) => {
                // Get the current progress based on actual audio sample position
                let progress = position_guard.progress();
                let position = position_guard.position();
                let duration = if position_guard.total_samples > 0 {
                    Some(position_guard.duration())
                } else {
                    None
                };
                
                // Determine if we've reached the end
                let should_stop = progress >= 1.0;
                
                Some((should_stop, progress, position, duration))
            },
            Err(_) => None
        };
        
        if let Some((should_stop, progress, position, duration)) = progress_info {
            // Update the player state with accurate timing information
            if let Ok(mut state) = self.state.lock() {
                state.progress = progress;
                state.position = Some(position);
                state.duration = duration;
            }

            if should_stop {
                self.stop();
            }
        }
    }

    pub fn seek(&mut self, position: f32) {
        // Seek to a specific position in the track (0.0 to 1.0)
        let position = position.max(0.0).min(1.0);
        
        // Add safer error handling for mutex lock
        if let Ok(position_guard) = self.playback_position.lock() {
            position_guard.seek(position);
            
            // Update the player state
            if let Ok(mut state) = self.state.lock() {
                state.progress = position;
            }
        }
    }
    
    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.max(0.0).min(1.0);
        
        // Add safer error handling for mutex locks
        if let Ok(mut vol) = self.volume.lock() {
            *vol = volume;
        }
        
        if let Ok(mut state) = self.state.lock() {
            state.volume = volume;
        }
    }

    pub fn get_state(&self) -> PlayerState {
        // Add safer error handling
        match self.state.lock() {
            Ok(state) => state.clone(),
            Err(_) => PlayerState::new() // Return default state if lock fails
        }
    }
    
    // Added method to toggle shuffle state
    pub fn toggle_shuffle_state(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.shuffle_enabled = !state.shuffle_enabled;
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

// Ring buffer for audio samples to allow smoother playback
struct AudioRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    samples_available: usize,
}

impl AudioRingBuffer {
    fn new(capacity: usize) -> Self {
        // Ensure minimum capacity to prevent issues with tiny buffers
        let capacity = capacity.max(1024);
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            samples_available: 0,
        }
    }
    
    fn write(&mut self, samples: &[f32]) -> usize {
        // Safety check for empty input
        if samples.is_empty() || self.capacity == 0 {
            return 0;
        }
        
        let mut written = 0;
        for &sample in samples {
            if self.samples_available >= self.capacity {
                break;
            }
            
            // Double-check bounds for extra safety
            if self.write_pos < self.buffer.len() {
                self.buffer[self.write_pos] = sample;
                self.write_pos = (self.write_pos + 1) % self.capacity;
                self.samples_available += 1;
                written += 1;
            }
        }
        
        written
    }
    
    fn read(&mut self, output: &mut [f32]) -> usize {
        // Safety check for empty output buffer
        if output.is_empty() || self.samples_available == 0 {
            return 0;
        }
        
        let to_read = output.len().min(self.samples_available);
        
        for i in 0..to_read {
            // Double-check bounds for extra safety
            if i < output.len() && self.read_pos < self.buffer.len() {
                output[i] = self.buffer[self.read_pos];
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
        
        // Update samples_available safely using saturating_sub to prevent underflow
        self.samples_available = self.samples_available.saturating_sub(to_read);
        to_read
    }
    
    fn available(&self) -> usize {
        self.samples_available
    }
}

// Improved resample function with better safety features
fn resample(input: &[f32], input_rate: u32, output_rate: u32, channels: usize) -> Vec<f32> {
    // Safety checks for invalid inputs
    if input.is_empty() || channels == 0 {
        return Vec::new();
    }
    
    if input_rate == output_rate {
        return input.to_vec();
    }
    
    // Ensure rates are valid to prevent division by zero
    if input_rate == 0 || output_rate == 0 {
        return input.to_vec();
    }
    
    let ratio = input_rate as f64 / output_rate as f64;
    let input_frames = input.len() / channels;
    
    // Safety check for empty input or zero ratio
    if input_frames == 0 || ratio <= 0.0 {
        return Vec::new();
    }
    
    let output_frames = (input_frames as f64 / ratio) as usize;
    
    // Pre-allocate with exact size for memory safety
    let mut output = Vec::with_capacity(output_frames * channels);
    
    for frame in 0..output_frames {
        let src_frame = frame as f64 * ratio;
        let src_frame_i = src_frame as usize;
        let fract = src_frame - src_frame_i as f64;
        
        // Guard against exceeding input bounds with an extra safety margin
        if src_frame_i >= input_frames.saturating_sub(1) {
            break;
        }
        
        for ch in 0..channels {
            // Extra bounds checking before indexing
            let curr_idx = src_frame_i * channels + ch;
            let next_idx = (src_frame_i + 1) * channels + ch;
            
            if curr_idx >= input.len() || next_idx >= input.len() {
                continue;
            }
            
            let curr = input[curr_idx];
            let next = input[next_idx];
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
    state_arc: Arc<Mutex<PlayerState>>,
    playback_position: Arc<Mutex<PlaybackPosition>>,
    volume_arc: Arc<Mutex<f32>>,
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
    let probed = match symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts) {
            Ok(p) => p,
            Err(e) => {
                error!("Error probing media: {}", e);
                return Err(anyhow!("Error probing media format: {}", e));
            }
        };
    
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
    
    let track = match format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL) {
            Some(t) => t,
            None => {
                error!("No supported audio track found in file");
                return Err(anyhow!("No supported audio track found"));
            }
        };
    
    info!("Found audio track: {}", track.codec_params.codec);
    
    let mut decoder = match symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts) {
            Ok(d) => d,
            Err(e) => {
                error!("Error creating decoder: {}", e);
                return Err(anyhow!("Error creating audio decoder: {}", e));
            }
        };
    
    let track_id = track.id;
    let channels = track.codec_params.channels.unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT);
    let channel_count = channels.count();
    let file_sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    
    info!("Audio track parameters: {} Hz, {} channels", file_sample_rate, channel_count);
    
    // Calculate total number of samples and duration with extra safety checks
    let total_samples = if let Some(n_frames) = track.codec_params.n_frames {
        // Prevent overflow by using checked multiplication
        match n_frames.checked_mul(channel_count as u64) {
            Some(samples) => samples,
            None => {
                // Fallback to a reasonable default if multiplication overflows
                warn!("Sample count calculation overflow, using fallback");
                file_sample_rate as u64 * channel_count as u64 * 300 // Assume 5 minutes
            }
        }
    } else {
        // Default to a reasonable amount if unknown
        file_sample_rate as u64 * channel_count as u64 * 300 // Assume 5 minutes
    };
    
    // Update the playback position with total samples
    if let Ok(mut position) = playback_position.lock() {
        position.set_total_samples(total_samples);
        position.sample_rate = file_sample_rate;
    }
    
    // Calculate duration safely
    let duration = {
        let sr = file_sample_rate.max(1) as f64; // Prevent division by zero
        let cc = channel_count.max(1) as f64;    // Prevent division by zero
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
    
    // Create a ring buffer for audio samples - smaller size for more responsive control
    // 100ms of audio at the output sample rate, minimum 1024 samples for safety
    let buffer_size = ((output_sample_rate as usize * channel_count as usize) / 10).max(1024);
    let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(buffer_size * 4)));
    let ring_buffer_stream = Arc::clone(&ring_buffer);
    
    // Flag to signal when audio needs more data
    let needs_data = Arc::new(AtomicBool::new(true));
    let needs_data_stream = Arc::clone(&needs_data);
    
    info!("Building audio output stream...");
    let stream = match device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // FIXED UNWIND SAFETY: Access the buffer inside a separate function
            // that doesn't involve catch_unwind or mutable references crossing boundaries
            
            // This method returns a Vec of samples and the count as owned values
            // that can safely cross the unwind boundary
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
    
    info!("Starting audio playback stream");
    if let Err(e) = stream.play() {
        error!("Failed to start audio stream: {}", e);
        return Err(anyhow!("Failed to start audio playback: {}", e));
    }
    
    info!("Audio stream started");
    let spec = SignalSpec::new(file_sample_rate, channels);
    // Explicitly set as usize to avoid type conversion issues
    let buffer_frames: usize = 2048; 
    let mut sample_buf = SampleBuffer::<f32>::new(buffer_frames as u64, spec);
    
    // Decoder loop - much more responsive with smaller buffers
    info!("Starting decode loop");
    let mut is_eof = false;
    let mut total_samples_processed = 0;
    let mut last_progress_update = Instant::now();
    
    while !is_eof {
        // Check stop flag more frequently
        if stop_flag.load(Ordering::SeqCst) {
            info!("Stop signal received; exiting decode loop");
            break;
        }
        
        // Check pause flag
        if pause_flag.load(Ordering::SeqCst) {
            // When paused, we just sleep briefly and check again
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        
        // Update playback progress regularly
        if last_progress_update.elapsed() >= Duration::from_millis(50) {
            // Update current position based on processed samples
            if let Ok(position) = playback_position.lock() {
                position.update_current_sample(total_samples_processed);
            }
            
            // Reset counter for next update
            total_samples_processed = 0;
            last_progress_update = Instant::now();
        }
        
        // Check if we need to fill the buffer
        if !needs_data.load(Ordering::Acquire) {
            // Buffer is still sufficiently full, sleep briefly and check again
            thread::sleep(Duration::from_millis(1));
            continue;
        }
        
        // We need to fill the buffer with more audio data
        needs_data.store(false, Ordering::Release);
        
        // Process just one packet at a time for more responsive control
        let packet_result: Result<symphonia::core::formats::Packet, symphonia::core::errors::Error> = match format.next_packet() {
            Ok(packet) => {
                if packet.track_id() != track_id {
                    continue;
                }
                Ok(packet)
            },
            Err(e) => {
                // Check if it's EOF or a real error
                if let symphonia::core::errors::Error::IoError(ref err) = e {
                    if err.kind() == std::io::ErrorKind::UnexpectedEof {
                        is_eof = true;
                        info!("End of file reached");
                        continue;
                    }
                }
                // Handle other errors by skipping this iteration
                warn!("Error reading packet: {}", e);
                continue;
            }
        };
        
        if let Ok(packet) = packet_result {
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Check if we need to resize the sample buffer - prevent buffer overrun
                    let decoded_frames = decoded.frames();
                    if sample_buf.capacity() < (decoded_frames * channel_count) as usize {
                        // Convert decoded_frames to usize for safe comparison
                        let needed_capacity = (decoded_frames * channel_count) as usize;
                        // No need for explicit conversion since both are usize
                        let new_capacity = needed_capacity.max(buffer_frames);
                        
                        // Create a new buffer with the larger capacity
                        sample_buf = SampleBuffer::<f32>::new(
                            new_capacity as u64, // Convert back to u64 for SampleBuffer::new
                            spec
                        );
                    }
                    
                    // Get interleaved samples with panic protection
                    sample_buf.copy_interleaved_ref(decoded);
                    let samples = sample_buf.samples();
                    
                    // Apply volume control with safer pre-allocation
                    let capacity = samples.len();
                    let mut volume_adjusted = Vec::with_capacity(capacity);
                    
                    let volume = match volume_arc.lock() {
                        Ok(v) => *v,
                        Err(_) => 0.8 // Default volume if lock fails
                    };
                    
                    // Apply volume safely
                    for &sample in samples {
                        volume_adjusted.push(sample * volume);
                    }
                    
                    // Protect against underflow by using saturating_add
                    let sample_count = volume_adjusted.len();
                    if sample_count > 0 {
                        total_samples_processed = total_samples_processed.saturating_add(sample_count);
                    }
                    
                    // Apply resampling if needed with extra safety checks
                    let output_samples = if file_sample_rate != output_sample_rate && !volume_adjusted.is_empty() {
                        resample(&volume_adjusted, file_sample_rate, output_sample_rate, channel_count)
                    } else {
                        volume_adjusted
                    };
                    
                    // Only try to write if we have samples and can get the buffer lock
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
            // End of file or error
            is_eof = true;
        }
    }
    
    info!("Decoding complete");
    
    // Wait for buffer to empty before returning (max 1 second) with safe error handling
    let wait_start = Instant::now();
    while wait_start.elapsed() < Duration::from_secs(1) {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        
        let buffer_empty = match ring_buffer.lock() {
            Ok(buffer) => buffer.available() == 0,
            Err(_) => true // Assume empty if we can't get the lock
        };
        
        if buffer_empty {
            break;
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    info!("Playback finished");
    Ok(())
}