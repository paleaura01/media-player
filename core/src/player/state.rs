use std::time::Duration;
use serde::{Serialize, Deserialize};

// Add #[derive(Serialize, Deserialize)] to make serialization work
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub status: PlaybackStatus,
    pub current_track: Option<String>,
    pub progress: f32,
    pub volume: f32,
    #[serde(skip)]  // Skip serializing these fields
    pub duration: Option<Duration>,
    #[serde(skip)]
    pub position: Option<Duration>,
    pub shuffle_enabled: bool,
    pub track_completed: bool,  // Added track completion flag
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
            shuffle_enabled: false,
            track_completed: false,  // Initialize to false
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}