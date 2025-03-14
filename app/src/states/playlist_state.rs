// app/src/states/playlist_state.rs
use std::time::{Instant, Duration};
use crate::ui::playlist_view::PlaylistAction;
use core::{Action, PlaylistAction as CorePlaylistAction};

pub struct PlaylistViewState {
    pub editing_playlist: Option<u32>,
    pub edit_value: String,
    pub last_click: Option<(u32, Instant)>,
    pub hovered_playlist_id: Option<u32>,
}

impl PlaylistViewState {
    pub fn new() -> Self {
        Self {
            editing_playlist: None,
            edit_value: String::new(),
            last_click: None,
            hovered_playlist_id: None,
        }
    }
    
    // Handle actions from playlist view and convert to core actions
    pub fn handle_action(&mut self, action: PlaylistAction) -> Action {
        match action {
            PlaylistAction::Select(id) => {
                // Log selection action for debugging
                println!("UI selecting playlist ID: {}", id);
                
                // Check for double click
                let now = Instant::now();
                if let Some((last_id, last_time)) = self.last_click {
                    if last_id == id && now.duration_since(last_time) < Duration::from_millis(500) {
                        // Double click detected - get name from the playlist
                        return self.handle_action(PlaylistAction::StartEditing(id, "".to_string()));
                    }
                }
                self.last_click = Some((id, now));
                
                // Return the core action for selection
                Action::Playlist(CorePlaylistAction::Select(id))
            },
            PlaylistAction::StartEditing(id, name) => {
                self.editing_playlist = Some(id);
                self.edit_value = name;
                // Just select the playlist but we're in editing mode now
                Action::Playlist(CorePlaylistAction::Select(id))
            },
            PlaylistAction::EditingName(value) => {
                self.edit_value = value;
                // No core action needed, just update our internal state
                Action::Playlist(CorePlaylistAction::Select(self.editing_playlist.unwrap_or(0)))
            },
            PlaylistAction::FinishEditing => {
                if let Some(id) = self.editing_playlist {
                    let name = self.edit_value.clone();
                    self.editing_playlist = None;
                    // Apply the rename action when editing is finished
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
                // Clear editing state if we're deleting the playlist being edited
                if Some(id) == self.editing_playlist {
                    self.editing_playlist = None;
                }
                
                println!("Sending delete action for playlist ID: {}", id);
                // Return the core action for deletion
                Action::Playlist(CorePlaylistAction::Delete(id))
            },
            PlaylistAction::HoverPlaylist(id) => {
                // Update the hovered playlist ID but don't affect selection
                self.hovered_playlist_id = id;
                // No action needed, just UI state update
                Action::Playlist(CorePlaylistAction::None)
            },
            PlaylistAction::None => {
                // Don't change selection state on None action
                Action::Playlist(CorePlaylistAction::None)
            },
        }
    }
    
    // Check if a particular playlist is being edited
    pub fn is_editing(&self, id: u32) -> bool {
        self.editing_playlist == Some(id)
    }
    
    // Get the current edit value
    pub fn edit_value(&self) -> &str {
        &self.edit_value
    }
}