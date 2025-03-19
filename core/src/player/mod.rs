// core/src/player/mod.rs
pub mod actions;
pub mod state;

// Re-export these types for backward compatibility
pub use self::state::{PlayerState, PlaybackStatus};

use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::{Result, anyhow};
use log::{info, error};

use crate::audio::position::PlaybackPosition;
use std::path::Path;

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
            match crate::audio::decoder::play_audio_file(&path_string, pause_flag, stop_flag, state_arc, playback_position, volume) {
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