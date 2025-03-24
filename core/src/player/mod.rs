// core/src/player/mod.rs

pub mod actions;
pub mod state;

pub use self::state::{PlayerState, PlaybackStatus};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use log::{info, error};

use crate::audio::position::PlaybackPosition;
use std::path::Path;

pub struct Player {
    pub state: Arc<Mutex<PlayerState>>,
    pause_flag: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    pub playback_position: Arc<Mutex<PlaybackPosition>>,
    volume: Arc<Mutex<f32>>,
    playback_thread: Option<thread::JoinHandle<()>>,
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
            playback_position: Arc::new(Mutex::new(PlaybackPosition::new(44100))),
            volume: Arc::new(Mutex::new(0.8)),
            playback_thread: None,
        }
    }

    pub fn play(&mut self, path: &str) -> Result<()> {
        info!("Player::play called with path: {}", path);

        if !Path::new(path).exists() {
            error!("File does not exist: {}", path);
            return Err(anyhow!("File not found: {}", path));
        }

        self.stop();
        self.stop_flag.store(false, Ordering::SeqCst);
        self.pause_flag.store(false, Ordering::SeqCst);

        // Reset playback position
        if let Ok(pos) = self.playback_position.lock() {
            pos.reset();
        }

        if let Ok(mut st) = self.state.lock() {
            st.status = PlaybackStatus::Playing;
            st.current_track = Some(path.to_string());
            st.progress = 0.0;
        }

        let path_string = path.to_string();
        let pause_flag = Arc::clone(&self.pause_flag);
        let stop_flag = Arc::clone(&self.stop_flag);
        let state_arc = Arc::clone(&self.state);
        let playback_position = Arc::clone(&self.playback_position);
        let volume = Arc::clone(&self.volume);

        self.playback_thread = Some(thread::spawn(move || {
            match crate::audio::decoder::play_audio_file(
                &path_string,
                pause_flag,
                stop_flag,
                state_arc,
                playback_position,
                volume,
            ) {
                Ok(_) => info!("Playback finished successfully"),
                Err(e) => error!("Playback error: {}", e),
            }
        }));

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Ok(mut st) = self.state.lock() {
            if st.status == PlaybackStatus::Playing {
                info!("Pausing playback");
                self.pause_flag.store(true, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(5));
                st.status = PlaybackStatus::Paused;
            }
        }
    }

    pub fn resume(&mut self) {
        if let Ok(mut st) = self.state.lock() {
            if st.status == PlaybackStatus::Paused {
                info!("Resuming playback");
                self.pause_flag.store(false, Ordering::SeqCst);
                st.status = PlaybackStatus::Playing;
            }
        }
    }

    pub fn stop(&mut self) {
        if let Ok(mut st) = self.state.lock() {
            if st.status != PlaybackStatus::Stopped {
                info!("Stopping playback");
                self.stop_flag.store(true, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(10));
                st.status = PlaybackStatus::Stopped;
                st.current_track = None;
                st.progress = 0.0;
                st.position = None;
                st.duration = None;

                if let Ok(pos) = self.playback_position.lock() {
                    pos.reset();
                }

                if let Some(handle) = self.playback_thread.take() {
                    let _ = handle.join();
                }
            }
        }
    }

    /// Called frequently (e.g. in the application `update` loop) to keep the UI in sync.
    pub fn update_progress(&mut self) {
        if let Ok(pos) = self.playback_position.lock() {
            let new_progress = pos.progress();
            let new_time = pos.position();
            let dur = pos.duration();

            if let Ok(mut st) = self.state.lock() {
                st.progress = new_progress;
                st.position = Some(new_time);
                st.duration = Some(dur);
            }
        }
    }

    /// (Deprecated) Directly seek to a fraction. 
    pub fn seek(&mut self, fraction: f32) {
        log::warn!("Player::seek({:.4}) is deprecated in favor of request_seek usage", fraction);
        if let Ok(pos) = self.playback_position.lock() {
            pos.seek(fraction);
        }
        if let Ok(mut st) = self.state.lock() {
            st.progress = fraction;
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        let volume = vol.clamp(0.0, 1.0);
        if let Ok(mut v) = self.volume.lock() {
            *v = volume;
        }

        if let Ok(mut st) = self.state.lock() {
            st.volume = volume;
        }
        info!("Volume set to {:.2}", volume);
    }

    pub fn get_state(&self) -> PlayerState {
        match self.state.lock() {
            Ok(s) => s.clone(),
            Err(e) => {
                error!("Failed to get player state: {}", e);
                PlayerState::new()
            }
        }
    }

    pub fn toggle_shuffle_state(&mut self) {
        if let Ok(mut st) = self.state.lock() {
            st.shuffle_enabled = !st.shuffle_enabled;
            info!("Shuffle toggled to: {}", st.shuffle_enabled);
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        info!("Dropping Player, stopping playback");
        self.stop();
    }
}
