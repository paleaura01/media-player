// core/src/lib.rs
pub mod audio;
pub mod player;
pub mod playlist;
pub mod library;

// Re-export key types for convenience
pub use player::state::{PlayerState, PlaybackStatus};
pub use player::actions::PlayerAction;
pub use player::Player;
pub use playlist::{PlaylistAction, PlaylistState, Playlist, Track};
pub use library::{LibraryAction, LibraryState};

#[derive(Debug, Clone)]
pub enum Action {
    Player(PlayerAction),
    Playlist(PlaylistAction),
    Library(LibraryAction),
}