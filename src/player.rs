// src/player.rs
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

use anyhow::Result;

use crate::audio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    state: PlayerState,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    playback_thread: Option<thread::JoinHandle<()>>,
}

impl Player {
    pub fn new() -> Self {
        log::info!("Initializing Media Player...");
        let pause_flag = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::new(AtomicBool::new(false));
        
        Self {
            state: PlayerState::Stopped,
            pause_flag,
            stop_flag,
            playback_thread: None,
        }
    }

    pub fn play(&mut self, path: &str) -> Result<()> {
        // Reset flags and state
        self.stop_flag.store(false, Ordering::SeqCst);
        self.pause_flag.store(false, Ordering::SeqCst);
        self.state = PlayerState::Playing;
        
        // Create clones of the flags for the playback thread
        let path_string = path.to_string();
        let pause_flag = Arc::clone(&self.pause_flag);
        let stop_flag = Arc::clone(&self.stop_flag);
        
        // Start playback in a new thread
        self.playback_thread = Some(thread::spawn(move || {
            match audio::play_audio_file(&path_string, pause_flag, stop_flag) {
                Ok(_) => log::info!("Playback finished successfully"),
                Err(e) => log::error!("Playback error: {}", e),
            }
        }));
        
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing {
            self.pause_flag.store(true, Ordering::SeqCst);
            self.state = PlayerState::Paused;
            log::info!("Playback paused");
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlayerState::Paused {
            self.pause_flag.store(false, Ordering::SeqCst);
            self.state = PlayerState::Playing;
            log::info!("Playback resumed");
        }
    }

    pub fn stop(&mut self) {
        if self.state != PlayerState::Stopped {
            self.stop_flag.store(true, Ordering::SeqCst);
            self.state = PlayerState::Stopped;
            log::info!("Playback stopped");
            
            // Wait for playback thread to finish
            if let Some(handle) = self.playback_thread.take() {
                let _ = handle.join();
            }
        }
    }

    pub fn wait_until_finished(&mut self) {
        if let Some(handle) = self.playback_thread.take() {
            let _ = handle.join();
            self.state = PlayerState::Stopped;
        }
    }

    pub fn get_state(&self) -> PlayerState {
        self.state
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}