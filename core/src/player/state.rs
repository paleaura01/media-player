use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub status: PlaybackStatus,
    pub current_track: Option<String>,
    pub progress: f32,
    pub volume: f32,
    #[serde(skip)]
    pub duration: Option<Duration>,
    #[serde(skip)]
    pub position: Option<Duration>,
    pub shuffle_enabled: bool,
    pub track_completed: bool,
    // Network playback fields
    pub network_buffering: bool,
    pub buffer_progress: f32,
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
            track_completed: false,
            network_buffering: false,
            buffer_progress: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}