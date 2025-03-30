// app/src/states/playlist_state.rs
use std::time::{Instant, Duration};
use crate::ui::playlist_view::PlaylistAction;
use core::{Action, PlaylistAction as CorePlaylistAction};
use crate::ui::library_view::LibraryMessage; 

pub struct PlaylistViewState {
    pub editing_playlist: Option<u32>,
    pub edit_value: String,
    pub last_click: Option<(u32, Instant)>,
    pub is_seeking: bool, // Added field for seek tracking
}

impl PlaylistViewState {
    pub fn new() -> Self {
        Self {
            editing_playlist: None,
            edit_value: String::new(),
            last_click: None,
            is_seeking: false, // Initialize to false
        }
    }
    
    pub fn handle_action(&mut self, action: PlaylistAction) -> Action {
        match action {
            PlaylistAction::Select(id) => {
                println!("UI selecting playlist ID: {}", id);
                
                let now = Instant::now();
                if let Some((last_id, last_time)) = self.last_click {
                    if last_id == id && now.duration_since(last_time) < Duration::from_millis(500) {
                        // Double click
                        return self.handle_action(PlaylistAction::StartEditing(id, "".to_string()));
                    }
                }
                self.last_click = Some((id, now));
                
                Action::Playlist(CorePlaylistAction::Select(id))
            },
            PlaylistAction::StartEditing(id, name) => {
                self.editing_playlist = Some(id);
                self.edit_value = name;
                Action::Playlist(CorePlaylistAction::Select(id))
            },
            PlaylistAction::EditingName(value) => {
                self.edit_value = value;
                Action::Playlist(CorePlaylistAction::Select(self.editing_playlist.unwrap_or(0)))
            },
            PlaylistAction::FinishEditing => {
                if let Some(id) = self.editing_playlist {
                    let name = self.edit_value.clone();
                    self.editing_playlist = None;
                    Action::Playlist(CorePlaylistAction::Rename(id, name))
                } else {
                    Action::Playlist(CorePlaylistAction::Select(0))
                }
            },
            PlaylistAction::Create(name) => {
                println!("Creating new playlist: {}", name);
                Action::Playlist(CorePlaylistAction::Create(name))
            },
            PlaylistAction::Delete(id) => {
                if Some(id) == self.editing_playlist {
                    self.editing_playlist = None;
                }
                println!("Sending delete action for playlist ID: {}", id);
                Action::Playlist(CorePlaylistAction::Delete(id))
            },
            
            PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                println!("Request to play track {} from playlist {}", track_idx, playlist_id);
                Action::Playlist(CorePlaylistAction::PlayTrack(playlist_id, track_idx))
            },
            
            // Fixed implementation for BatchAddTracks
            PlaylistAction::BatchAddTracks(playlist_id, tracks) => {
                println!("Request to add {} tracks to playlist {}", tracks.len(), playlist_id);
                Action::Playlist(CorePlaylistAction::BatchAddTracks(playlist_id, tracks))
            },
            
            // Fix for the Seek variant
            PlaylistAction::Seek(position) => {
                println!("██ DEBUG: PlaylistAction::Seek({:.4}) in playlist_view_state.rs", position);
                println!("██ DEBUG: Converting to Action::Player(PlayerAction::Seek)");
                Action::Player(core::PlayerAction::Seek(position))
            },
            
            // UpdateProgress variant
            PlaylistAction::UpdateProgress(_pos) => {
                // Just for UI updates during dragging, no actual seeking
                Action::Playlist(CorePlaylistAction::None)
            },
            
            PlaylistAction::PlayerControl(player_action) => {
                println!("Player control action: {:?}", player_action);
                Action::Player(player_action)
            },
            PlaylistAction::None => {
                Action::Playlist(CorePlaylistAction::None)
            },
            
            // Handler for RemoveTrack
            PlaylistAction::RemoveTrack(playlist_id, track_idx) => {
                println!("Requesting to remove track {} from playlist {}", track_idx, playlist_id);
                Action::Playlist(CorePlaylistAction::RemoveTrack(playlist_id, track_idx))
            },

            PlaylistAction::Library(library_action) => {
                // Convert library messages to core library actions
                match library_action {
                    LibraryMessage::AddMusicFolder => {
                        Action::Library(core::LibraryAction::StartScan)
                    },
                    LibraryMessage::ToggleView => {
                        // For view toggle, we can just return a no-op action
                        Action::Library(core::LibraryAction::None)
                    }
                }
            },
        }
    }
    
    pub fn is_editing(&self, id: u32) -> bool {
        self.editing_playlist == Some(id)
    }
    
    pub fn edit_value(&self) -> &str {
        &self.edit_value
    }
}