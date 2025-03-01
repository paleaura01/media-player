pub mod player;
pub mod playlist;
pub mod library;

pub use player::{Player, PlayerState, PlaybackStatus, PlayerAction};
pub use playlist::{PlaylistState, Playlist, Track, PlaylistAction};
pub use library::{LibraryState, LibraryAction};

// Main action enum used for UI events
#[derive(Debug, Clone)]
pub enum Action {
    Player(PlayerAction),
    Playlist(PlaylistAction),
    Library(LibraryAction),
}