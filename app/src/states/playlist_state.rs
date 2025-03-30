// app/src/states/playlist_state.rs
use std::time::{Instant, Duration};
use log::{debug, info, warn};
use crate::ui::playlist_view::PlaylistAction;
use core::{Action, PlaylistAction as CorePlaylistAction};
use crate::ui::library_view::LibraryMessage;

#[derive(Debug)]
pub struct PlaylistViewState {
    pub editing_playlist: Option<u32>,
    pub edit_value: String,
    pub last_click: Option<(u32, Instant)>,
    pub is_seeking: bool,
}

impl PlaylistViewState {
    pub fn new() -> Self {
        Self {
            editing_playlist: None,
            edit_value: String::new(),
            last_click: None,
            is_seeking: false,
        }
    }

    pub fn handle_action(&mut self, action: PlaylistAction) -> Action {
        match action {
            PlaylistAction::Select(id) => {
                debug!("UI selecting playlist ID: {}", id);
                let now = Instant::now();
                let mut core_action = CorePlaylistAction::Select(id);

                // Double-click detection for editing
                if let Some((last_id, last_time)) = self.last_click {
                    if last_id == id && now.duration_since(last_time) < Duration::from_millis(500) {
                        info!("Double-click detected on playlist ID: {}, starting edit.", id);
                        self.editing_playlist = Some(id);
                        self.edit_value = "".to_string();
                        self.last_click = None;
                        core_action = CorePlaylistAction::None;
                    } else {
                        self.last_click = Some((id, now));
                    }
                } else {
                    self.last_click = Some((id, now));
                }
                
                // Fix comparison with PlaylistAction
                if self.editing_playlist.is_some() && self.editing_playlist != Some(id) {
                    if let CorePlaylistAction::None = core_action {
                        // No action needed
                    } else {
                        debug!("Selection changed, canceling edit state.");
                        self.editing_playlist = None;
                        self.edit_value.clear();
                    }
                }

                Action::Playlist(core_action)
            },
            PlaylistAction::StartEditing(id, name) => {
                info!("UI StartEditing for playlist ID: {}", id);
                self.editing_playlist = Some(id);
                self.edit_value = name;
                self.last_click = None;
                Action::Playlist(CorePlaylistAction::Select(id))
            },
            PlaylistAction::EditingName(value) => {
                if self.editing_playlist.is_some() {
                    self.edit_value = value;
                }
                Action::Playlist(CorePlaylistAction::None)
            },
            PlaylistAction::FinishEditing => {
                info!("UI FinishEditing");
                if let Some(id) = self.editing_playlist.take() {
                    let name_to_save = self.edit_value.trim().to_string();
                    self.edit_value.clear();
                    if !name_to_save.is_empty() {
                        info!("Finishing edit for ID {}, saving name '{}'", id, name_to_save);
                        Action::Playlist(CorePlaylistAction::Rename(id, name_to_save))
                    } else {
                        warn!("Edit finished with empty name for ID {}, canceling rename.", id);
                        Action::Playlist(CorePlaylistAction::None)
                    }
                } else {
                    warn!("FinishEditing called but no playlist was being edited.");
                    Action::Playlist(CorePlaylistAction::None)
                }
            },
            // Rest of the method unchanged...
            // Just including sections needed for fixes
            
            PlaylistAction::Create(name) => {
                info!("UI requesting Create Playlist: {}", name);
                self.editing_playlist = None;
                self.edit_value.clear();
                Action::Playlist(CorePlaylistAction::Create(name))
            },
            PlaylistAction::Delete(id) => {
                info!("UI requesting Delete Playlist ID: {}", id);
                if self.editing_playlist == Some(id) {
                    self.editing_playlist = None;
                    self.edit_value.clear();
                }
                Action::Playlist(CorePlaylistAction::Delete(id))
            },
            // Other cases...
            _ => {
                // Default implementation for other actions
                match action {
                    PlaylistAction::None => Action::Playlist(CorePlaylistAction::None),
                    PlaylistAction::PlayTrack(pid, tid) => Action::Playlist(CorePlaylistAction::PlayTrack(pid, tid)),
                    PlaylistAction::RemoveTrack(pid, idx) => Action::Playlist(CorePlaylistAction::RemoveTrack(pid, idx)),
                    PlaylistAction::BatchAddTracks(pid, tracks) => Action::Playlist(CorePlaylistAction::BatchAddTracks(pid, tracks)),
                    PlaylistAction::Seek(pos) => Action::Player(core::PlayerAction::Seek(pos)),
                    PlaylistAction::UpdateProgress(_) => Action::Playlist(CorePlaylistAction::None),
                    PlaylistAction::PlayerControl(action) => Action::Player(action),
                    PlaylistAction::Library(action) => match action {
                        LibraryMessage::AddMusicFolder => Action::Library(core::LibraryAction::None),
                        LibraryMessage::ToggleView => Action::Library(core::LibraryAction::None),
                    },
                    _ => Action::Playlist(CorePlaylistAction::None), // Catch-all for cases already handled
                }
            }
        }
    }

    pub fn is_editing(&self, id: u32) -> bool {
        self.editing_playlist == Some(id)
    }

    pub fn edit_value(&self) -> &str {
        &self.edit_value
    }
}