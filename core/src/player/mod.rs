use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::Result;
use log::{info, error, debug, warn};

pub mod state;
pub mod actions;

use crate::audio;
use crate::audio::position::PlaybackPosition;
use crate::player::state::{PlayerState, PlaybackStatus};

pub struct Player {
    pub state: Arc<Mutex<PlayerState>>,
    pub pause_flag: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
    pub playback_position: Arc<Mutex<PlaybackPosition>>,
    pub volume: Arc<Mutex<f32>>,
    pub playback_thread: Option<thread::JoinHandle<()>>,
    pub track_completed_signal: bool,
    track_completed_flag: Arc<AtomicBool>,
    is_network_path: bool,
}

impl Player {
    pub fn new() -> Self {
        info!("Initializing Media Player with FFmpeg...");
        
        // Initialize FFmpeg
        if let Err(e) = crate::audio::decoder::initialize_ffmpeg() {
            error!("Failed to initialize FFmpeg: {}", e);
        } else {
            // Log supported formats
            let formats = crate::audio::decoder::get_supported_extensions();
            info!("Supported audio formats: {}", formats.join(", "));
        }
        
        let pause_flag = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let track_completed_flag = Arc::new(AtomicBool::new(false));

        Self {
            state: Arc::new(Mutex::new(PlayerState::new())),
            pause_flag,
            stop_flag,
            playback_position: Arc::new(Mutex::new(PlaybackPosition::new(44100))),
            volume: Arc::new(Mutex::new(0.8)),
            playback_thread: None,
            track_completed_signal: false,
            track_completed_flag,
            is_network_path: false,
        }
    }
    
    // Simple no-op method to maintain backward compatibility
    pub fn configure_network(&mut self, _buffer_size: usize, _prebuffer_seconds: u64) {
        // This method intentionally does nothing now that we're using direct streaming
        info!("Network buffering disabled, using direct streaming mode");
    }
    
    pub fn clear_audio_buffers(&self) {
        // Signal the audio thread to clear its buffers
        if let Ok(mut lock) = self.playback_position.lock() {
            lock.clear_buffers = true;
            debug!("Audio buffer clear request set");
        }
    }

    pub fn play(&mut self, path: &str) -> Result<()> {
        // Stop any current playback first
        self.stop();
        
        info!("Player::play({})", path);
        
        // Detect if path is a network path
        self.is_network_path = path.starts_with("\\\\") || path.contains("://");
        
        if self.is_network_path {
            info!("Network path detected, using direct streaming mode");
        }
        
        // Reset flags
        self.pause_flag.store(false, Ordering::SeqCst);
        self.stop_flag.store(false, Ordering::SeqCst);
        self.track_completed_flag.store(false, Ordering::SeqCst);
        
        // Update current track
        {
            if let Ok(mut state) = self.state.lock() {
                state.current_track = Some(path.to_string());
                state.status = PlaybackStatus::Playing;
                state.progress = 0.0;
                state.track_completed = false;
                
                // Set direct streaming mode for all files (no buffering)
                state.network_buffering = false;
                state.buffer_progress = 1.0; 
            }
        }
        
        // Reset playback position
        if let Ok(mut pos) = self.playback_position.lock() {
            pos.reset();
            // Clear any previous seek requests
            if let Some(flag) = &pos.seek_requested {
                flag.store(false, Ordering::SeqCst);
            }
            pos.clear_buffers = false;
        }
        
        let path_str = path.to_string();
        let state_arc = Arc::clone(&self.state);
        let pause_flag = Arc::clone(&self.pause_flag);
        let stop_flag = Arc::clone(&self.stop_flag);
        let playback_position = Arc::clone(&self.playback_position);
        let volume = Arc::clone(&self.volume);
        let track_completed = Arc::clone(&self.track_completed_flag);
        
        // Set up a channel for thread communication
        let (error_tx, error_rx) = std::sync::mpsc::channel();
        
        // Create the playback thread
        self.playback_thread = Some(thread::spawn(move || {
            // Create a separate thread to avoid blocking the main thread
            let error_tx_clone = error_tx.clone(); // Clone here before move
            
            let handle = thread::Builder::new()
                .name("audio_playback".to_string())
                .spawn(move || {
                    debug!("Starting audio playback thread");
                    
                    // Create a local copy for the closure
                    let state_arc_local = Arc::clone(&state_arc);
                    
                    // Use the simple implementation for all files
                    let result = audio::decoder::play_audio_file(
                        &path_str, 
                        pause_flag, 
                        stop_flag, 
                        state_arc_local,
                        playback_position, 
                        volume
                    );
                    
                    match result {
                        Ok(_) => {
                            // Playback completed successfully
                            track_completed.store(true, Ordering::SeqCst);
                            info!("Playback completed normally");
                        },
                        Err(e) => {
                            error!("Playback error: {}", e);
                            // Send error back to main thread
                            let _ = error_tx.send(e.to_string());
                            
                            // Update player state to stopped on error
                            if let Ok(mut state) = state_arc.lock() {
                                state.status = PlaybackStatus::Stopped;
                                state.network_buffering = false;
                            }
                        }
                    }
                });
                
            // Handle thread creation failure
            if let Err(e) = handle {
                error!("Failed to create playback thread: {}", e);
                let _ = error_tx_clone.send(format!("Thread creation failed: {}", e));
            }
        }));
        
        // Check for immediate errors
        match error_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(err_msg) => {
                error!("Failed to start playback: {}", err_msg);
                return Err(anyhow::anyhow!("Playback error: {}", err_msg));
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No immediate error, playback started
                info!("Started playback successfully");
            },
            Err(e) => {
                warn!("Error checking playback status: {}", e);
                // Continue anyway
            }
        }
        
        Ok(())
    }

    pub fn pause(&mut self) {
        info!("Player::pause()");
        self.pause_flag.store(true, Ordering::SeqCst);
        
        if let Ok(mut state) = self.state.lock() {
            state.status = PlaybackStatus::Paused;
        }
    }

    pub fn resume(&mut self) {
        info!("Player::resume()");
        self.pause_flag.store(false, Ordering::SeqCst);
        
        if let Ok(mut state) = self.state.lock() {
            state.status = PlaybackStatus::Playing;
        }
    }

    pub fn stop(&mut self) {
        info!("Player::stop()");
        self.stop_flag.store(true, Ordering::SeqCst);
        
        // Wait for playback thread to terminate
        if let Some(handle) = self.playback_thread.take() {
            // Don't wait forever - use a timeout
            let wait_start = Instant::now();
            while wait_start.elapsed() < Duration::from_secs(2) {
                if handle.is_finished() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            
            // If still not finished, just proceed
            if !handle.is_finished() {
                warn!("Playback thread did not terminate cleanly");
            }
        }
        
        // Reset flags for next playback
        self.stop_flag.store(false, Ordering::SeqCst);
        self.pause_flag.store(false, Ordering::SeqCst);
        
        // Update state
        if let Ok(mut state) = self.state.lock() {
            state.status = PlaybackStatus::Stopped;
            state.network_buffering = false;
        }
    }

    pub fn seek(&mut self, position: f32) {
        debug!("Player::seek({})", position);
        
        // Clamp position between 0 and 1
        let pos = position.clamp(0.0, 1.0);
        
        // Request seek in the playback position
        if let Ok(mut playback_pos) = self.playback_position.lock() {
            playback_pos.request_seek(pos);
        }
        
        // Also update the UI state immediately for better responsiveness
        if let Ok(mut state) = self.state.lock() {
            state.progress = pos;
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        if let Ok(mut v) = self.volume.lock() {
            *v = vol;
        }
        
        if let Ok(mut state) = self.state.lock() {
            state.volume = vol;
        }
    }

    pub fn update_progress(&mut self) {
        // Check if track completed
        if self.track_completed_flag.load(Ordering::SeqCst) {
            // Reset the flag
            self.track_completed_flag.store(false, Ordering::SeqCst);
            // Set the signal for the main thread
            self.track_completed_signal = true;
        }
        
        // Update playback progress
        let mut progress = 0.0;
        let mut duration = None;
        let mut position = None;
        
        // Get progress from the playback position
        if let Ok(playback_pos) = self.playback_position.lock() {
            progress = playback_pos.progress();
            position = Some(playback_pos.position());
            duration = Some(playback_pos.duration());
        }
        
        // Update the player state
        if let Ok(mut state) = self.state.lock() {
            state.progress = progress;
            
            // Update duration and position
            if let Some(dur) = duration {
                state.duration = Some(dur);
            }
            state.position = position;
            
            // Update track completion state
            if self.track_completed_signal {
                state.track_completed = true;
            }
        }
    }

    pub fn get_state(&self) -> PlayerState {
        if let Ok(state) = self.state.lock() {
            state.clone()
        } else {
            PlayerState::new()
        }
    }
}