// app/src/states/app_state.rs
// This file handles the core application state and actions

use std::path::PathBuf;
use log::{debug, error, info};
use core::{Action, PlayerAction, PlaylistAction, LibraryAction, Track, Player, PlayerState, PlaylistState, LibraryState};
use crate::states::playlist_state::PlaylistViewState;
use rand::Rng; // For picking random track if shuffle is on

pub struct MediaPlayer {
    pub player: Player,
    pub player_state: PlayerState,
    pub playlists: PlaylistState,
    pub library: LibraryState,
    pub data_dir: PathBuf,
    pub playlist_view_state: PlaylistViewState,
}

impl std::fmt::Debug for MediaPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaPlayer")
            .field("player", &"<player>")
            .field("player_state", &self.player_state)
            .field("playlists", &self.playlists)
            .field("library", &self.library)
            .field("data_dir", &self.data_dir)
            .finish()
    }
}

impl Default for MediaPlayer {
    fn default() -> Self {
        let data_dir = PathBuf::from("data");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        }

        let playlists_path = data_dir.join("playlists.json");
        let playlists = if playlists_path.exists() {
            match PlaylistState::load_from_file(&playlists_path) {
                Ok(pl) => pl,
                Err(e) => {
                    error!("Failed to load playlists: {}", e);
                    PlaylistState::new()
                }
            }
        } else {
            PlaylistState::new()
        };

        let player = Player::new();
        let mut player_state = player.get_state();
        // Make sure shuffle starts off
        player_state.shuffle_enabled = false;

        info!("MediaPlayer default state created");

        Self {
            player,
            player_state,
            playlists,
            library: LibraryState::new(),
            data_dir,
            playlist_view_state: PlaylistViewState::new(),
        }
    }
}

impl MediaPlayer {
    pub fn handle_action(&mut self, action: Action) {
        debug!("Handling Action: {:?}", action);
        match action {
            Action::Player(act) => self.handle_player_action(act),
            Action::Playlist(act) => self.handle_playlist_action(act),
            Action::Library(act) => self.handle_library_action(act),
        }
        // Always update the player state after handling actions
        self.player_state = self.player.get_state();
    }
    
    fn handle_player_action(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Play(path) => {
                // When playing a track, we pass the full path to the player
                info!("Attempting to play file: {}", path);
                if let Err(e) = self.player.play(&path) {
                    error!("Failed to play: {}", e);
                } else {
                    info!("Started playback successfully");
                }
            }
            PlayerAction::Pause => self.player.pause(),
            PlayerAction::Resume => self.player.resume(),
            PlayerAction::Stop => self.player.stop(),
            PlayerAction::SetVolume(v) => self.player.set_volume(v),
            PlayerAction::Seek(pos) => self.player.seek(pos),
            PlayerAction::Shuffle => {
                // Toggle shuffle mode
                self.player_state.shuffle_enabled = !self.player_state.shuffle_enabled;
                if self.player_state.shuffle_enabled {
                    info!("Shuffle enabled");
                    // Make sure shuffle is reflected in player state
                    if let Ok(mut state) = self.player.state.lock() {
                        state.shuffle_enabled = true;
                    }
                } else {
                    info!("Shuffle disabled");
                    // Make sure shuffle is reflected in player state
                    if let Ok(mut state) = self.player.state.lock() {
                        state.shuffle_enabled = false;
                    }
                }
            },
            PlayerAction::NextTrack => {
                // Code for next track with shuffle consideration
                info!("Next track button pressed");
                
                if let Some(idx) = self.playlists.selected {
                    if idx < self.playlists.playlists.len() {
                        let playlist = &self.playlists.playlists[idx];
                        
                        if self.player_state.shuffle_enabled {
                            // If shuffle is enabled, select a random track
                            if !playlist.tracks.is_empty() {
                                let random_idx = rand::thread_rng().gen_range(0..playlist.tracks.len());
                                
                                let track = &playlist.tracks[random_idx];
                                info!("Playing random track: {}", track.path);
                                self.handle_action(core::Action::Playlist(
                                    core::PlaylistAction::PlayTrack(playlist.id, random_idx)
                                ));
                            }
                        } else {
                            // Sequential next track logic
                            if let Some(current_track_path) = &self.player_state.current_track {
                                let current_idx = playlist.tracks.iter()
                                    .position(|track| &track.path == current_track_path);
                                    
                                if let Some(idx) = current_idx {
                                    let next_idx = (idx + 1) % playlist.tracks.len();
                                    
                                    let track = &playlist.tracks[next_idx];
                                    info!("Playing next track: {}", track.path);
                                    self.handle_action(core::Action::Playlist(
                                        core::PlaylistAction::PlayTrack(playlist.id, next_idx)
                                    ));
                                } else if !playlist.tracks.is_empty() {
                                    // Current track not in playlist, start with first
                                    self.handle_action(core::Action::Playlist(
                                        core::PlaylistAction::PlayTrack(playlist.id, 0)
                                    ));
                                }
                            } else if !playlist.tracks.is_empty() {
                                // No track playing yet
                                // If shuffle is ON, pick a random track
                                // If shuffle is OFF, start with the first track
                                if self.player_state.shuffle_enabled {
                                    let random_idx = rand::thread_rng().gen_range(0..playlist.tracks.len());
                                    let track = &playlist.tracks[random_idx];
                                    info!("(No track) shuffle => playing random track: {}", track.path);
                                    self.handle_action(core::Action::Playlist(
                                        core::PlaylistAction::PlayTrack(playlist.id, random_idx)
                                    ));
                                } else {
                                    info!("(No track) playing the first track in the playlist");
                                    self.handle_action(core::Action::Playlist(
                                        core::PlaylistAction::PlayTrack(playlist.id, 0)
                                    ));
                                }
                            }
                        }
                    }
                }
            },
            PlayerAction::PreviousTrack => {
                // Code for previous track (similar to NextTrack but going backwards)
                info!("Previous track button pressed");
                
                if let Some(idx) = self.playlists.selected {
                    if idx < self.playlists.playlists.len() {
                        let playlist = &self.playlists.playlists[idx];
                        
                        if let Some(current_track_path) = &self.player_state.current_track {
                            let current_idx = playlist.tracks.iter()
                                .position(|track| &track.path == current_track_path);
                                
                            if let Some(idx) = current_idx {
                                let prev_idx = if idx == 0 {
                                    playlist.tracks.len() - 1 // Wrap around to the end
                                } else {
                                    idx - 1
                                };
                                
                                let track = &playlist.tracks[prev_idx];
                                info!("Playing previous track: {}", track.path);
                                self.handle_action(core::Action::Playlist(
                                    core::PlaylistAction::PlayTrack(playlist.id, prev_idx)
                                ));
                            } else if !playlist.tracks.is_empty() {
                                // Current track not in playlist, start with last
                                let last_idx = playlist.tracks.len() - 1;
                                self.handle_action(core::Action::Playlist(
                                    core::PlaylistAction::PlayTrack(playlist.id, last_idx)
                                ));
                            }
                        } else if !playlist.tracks.is_empty() {
                            // No track playing, start with last
                            let last_idx = playlist.tracks.len() - 1;
                            self.handle_action(core::Action::Playlist(
                                core::PlaylistAction::PlayTrack(playlist.id, last_idx)
                            ));
                        }
                    }
                }
            },
        }
    }
    
    fn handle_playlist_action(&mut self, action: PlaylistAction) {
        match action {
            PlaylistAction::Create(name) => {
                self.playlists.create_playlist(name);
            },
            PlaylistAction::Delete(id) => {
                self.playlists.delete_playlist(id);
            },
            PlaylistAction::Select(id) => {
                if let Some(pos) = self.playlists.playlists.iter().position(|p| p.id == id) {
                    self.playlists.selected = Some(pos);
                }
            },
            PlaylistAction::Rename(id, new_name) => {
                let _ = self.playlists.rename_playlist(id, new_name);
            },
            PlaylistAction::AddTrack(playlist_id, track) => {
                if let Some(playlist) = self.playlists.get_playlist_mut(playlist_id) {
                    playlist.tracks.push(track);
                }
            },
            PlaylistAction::RemoveTrack(playlist_id, index) => {
                if let Some(playlist) = self.playlists.get_playlist_mut(playlist_id) {
                    if index < playlist.tracks.len() {
                        playlist.tracks.remove(index);
                    }
                }
            },
            PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                if let Some(playlist) = self.playlists.get_playlist(playlist_id) {
                    if track_idx < playlist.tracks.len() {
                        let track = &playlist.tracks[track_idx];
                        self.handle_action(core::Action::Player(
                            core::PlayerAction::Play(track.path.clone())
                        ));
                    }
                }
            },
            _ => {}
        }
    }
    
    fn handle_library_action(&mut self, action: LibraryAction) {
        match action {
            LibraryAction::AddScanDirectory(dir) => {
                self.library.scan_dirs.push(dir);
            }
            LibraryAction::RemoveScanDirectory(dir) => {
                self.library.scan_dirs.retain(|d| d != &dir);
            }
            LibraryAction::StartScan => {
                self.library.scanning = true;
                self.library.tracks.push(Track {
                    path: "sample1.mp3".to_string(),
                    title: Some("Sample Track 1".to_string()),
                    artist: Some("Artist 1".to_string()),
                    album: Some("Album 1".to_string()),
                });
                self.library.tracks.push(Track {
                    path: "sample2.mp3".to_string(),
                    title: Some("Sample Track 2".to_string()),
                    artist: Some("Artist 2".to_string()),
                    album: Some("Album 2".to_string()),
                });
                self.library.scanning = false;
            }
            LibraryAction::ImportFile(path) => {
                let filename = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
                self.library.tracks.push(Track {
                    path,
                    title: Some(filename),
                    artist: None,
                    album: None,
                });
            }
            LibraryAction::Search(_q) => {}
        }
    }
}