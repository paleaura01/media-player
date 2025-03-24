// core/src/player/actions.rs

#[derive(Clone, Debug)]
pub enum PlayerAction {
    Play(String),      // Path to file to play
    Pause,             // Pause playback
    Resume,            // Resume paused playback
    Stop,              // Stop playback completely
    SetVolume(f32),    // Set volume (0.0 to 1.0)
    Seek(f32),         // Seek to position (0.0 to 1.0)
    SkipForward(f32),  // Skip ahead by N seconds
    SkipBackward(f32), // Go back by N seconds
    Shuffle,           // Toggle shuffle mode
    NextTrack,         // Skip to next track
    PreviousTrack,     // Go to previous track
}