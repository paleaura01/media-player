// core/src/player/mod.rs
pub mod actions;
pub mod state;

// Re-export these types for backward compatibility
pub use self::state::{PlayerState, PlaybackStatus};

use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::{Result, anyhow};
use log::{info, error, debug};

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
        if let Ok(position) = self.playback_position.lock() {
            position.reset();
        }
        
        if let Ok(mut state) = self.state.lock() {
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
            match crate::audio::decoder::play_audio_file(
                &path_string,
                pause_flag,
                stop_flag,
                state_arc,
                playback_position,
                volume
            ) {
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
                
                if let Ok(position) = self.playback_position.lock() {
                    position.reset();
                }
                
                if let Some(handle) = self.playback_thread.take() {
                    let _ = handle.join();
                }
            }
        }
    }
    
    pub fn update_progress(&mut self) {
        // Get current progress from position tracker
        let progress_info = if let Ok(position) = self.playback_position.lock() {
            let progress = position.progress();
            let position_time = position.position();
            let duration_time = if position.total_samples > 0 {
                Some(position.duration())
            } else {
                None
            };
            
            let should_stop = progress >= 0.999; // Consider it complete at 99.9%
            
            Some((progress, position_time, duration_time, should_stop))
        } else {
            None
        };
        
        if let Some((progress, position_time, duration_time, should_stop)) = progress_info {
           // Update the player state with the current progress
           if let Ok(mut state) = self.state.lock() {
            state.progress = progress;
            state.position = Some(position_time);
            state.duration = duration_time;
        }
        
        // If playback has reached the end, stop
        if should_stop {
            self.stop();
        }
    }
}

/// Seeks playback to the specified position (0.0 = start, 1.0 = end)
/// UPDATED: Now uses explicit seek request mechanism
pub fn seek(&mut self, position: f32) {
    // Clamp position to valid range 0-1
    let position = position.max(0.0).min(1.0);
    
    println!("██ DEBUG: Player.seek({:.4}) called directly", position);
    
    // Update position tracker
    if let Ok(position_tracker) = self.playback_position.lock() {
        // Get current position for debugging
        let old_progress = position_tracker.progress();
        println!("██ DEBUG: Seeking from {:.4} to {:.4}", old_progress, position);
        
        // Set the new position
        position_tracker.seek(position);
        
        println!("██ DEBUG: Position tracker updated");
    } else {
        println!("██ DEBUG: Failed to get lock on position tracker");
    }
    
    // Update the UI state immediately for better responsiveness
    if let Ok(mut state) = self.state.lock() {
        let old_progress = state.progress;
        state.progress = position;
        println!("██ DEBUG: UI state progress updated from {:.4} to {:.4}", old_progress, position);
    } else {
        println!("██ DEBUG: Failed to get lock on player state");
    }
    
    println!("██ DEBUG: Seek operation completed in Player.seek()");
}

pub fn set_volume(&mut self, volume: f32) {
    // Ensure volume is properly clamped between 0.0 and 1.0
    let volume = volume.max(0.0).min(1.0);
    
    info!("Setting volume to: {:.4}", volume);
    
    // First update UI state to reflect change immediately
    if let Ok(mut state) = self.state.lock() {
        state.volume = volume;
    }
    
    // Now update the playback volume
    if let Ok(mut vol) = self.volume.lock() {
        *vol = volume;
    }
}

pub fn get_state(&self) -> PlayerState {
    // Add safer error handling
    match self.state.lock() {
        Ok(state) => state.clone(),
        Err(e) => {
            error!("Failed to get player state: {}", e);
            PlayerState::new() // Return default state if lock fails
        }
    }
}

/// Toggles shuffle state on or off
pub fn toggle_shuffle_state(&mut self) {
    if let Ok(mut state) = self.state.lock() {
        state.shuffle_enabled = !state.shuffle_enabled;
        info!("Shuffle mode toggled to: {}", state.shuffle_enabled);
    }
}
}

impl Drop for Player {
fn drop(&mut self) {
    info!("Player being dropped, stopping playback");
    self.stop();
}
}