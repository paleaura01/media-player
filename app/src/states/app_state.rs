// app/src/states/app_state.rs
// This file handles the core application state and actions

use std::path::PathBuf;
use log::{debug, error, info};
use core::{Action, PlayerAction, PlaylistAction, LibraryAction, Track, Player, PlayerState, PlaylistState, LibraryState};
use crate::states::playlist_state::PlaylistViewState;

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
        let player_state = player.get_state();

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
            PlayerAction::SetVolume(_v) => {}
            PlayerAction::Seek(_pos) => {}
        }
    }
    
    fn handle_playlist_action(&mut self, action: PlaylistAction) {
        match action {
            PlaylistAction::Create(name) => {
                info!("Creating playlist: {}", name);
                self.playlists.create_playlist(name);
                let path = self.data_dir.join("playlists.json");
                let _ = self.playlists.save_to_file(&path);
            }
            PlaylistAction::Select(id) => {
                info!("Selecting playlist with ID: {}", id);
                let idx = self.playlists.playlists.iter().position(|p| p.id == id);
                if let Some(idx) = idx {
                    info!("Found playlist at index: {}", idx);
                    self.playlists.selected = Some(idx);
                } else {
                    error!("Could not find playlist with ID: {}", id);
                    self.playlists.selected = None;
                }
            }
            PlaylistAction::Delete(id) => {
                info!("Deleting playlist with ID: {}", id);
                self.playlists.delete_playlist(id);
                let path = self.data_dir.join("playlists.json");
                if let Err(e) = self.playlists.save_to_file(&path) {
                    error!("Failed to save playlists after deletion: {}", e);
                } else {
                    info!("Successfully deleted playlist and saved changes");
                }
            }
            PlaylistAction::Rename(id, nm) => {
                if let Some(p) = self.playlists.get_playlist_mut(id) {
                    p.name = nm;
                    let path = self.data_dir.join("playlists.json");
                    let _ = self.playlists.save_to_file(&path);
                }
            }
            PlaylistAction::AddTrack(id, track) => {
                // Enhanced version that provides better logging and error handling for tracks
                if let Some(p) = self.playlists.get_playlist_mut(id) {
                    // Ensure we're preserving the full path of the track
                    info!("Adding track to playlist {}: {} (path: {})", 
                         id, track.title.as_deref().unwrap_or("Unknown"), track.path);
                    
                    // Add the track to the playlist
                    p.tracks.push(track);
                    
                    // Save the updated playlists to disk
                    let path = self.data_dir.join("playlists.json");
                    if let Err(e) = self.playlists.save_to_file(&path) {
                        error!("Failed to save playlists after adding track: {}", e);
                    } else {
                        info!("Successfully saved playlist with new track");
                    }
                } else {
                    error!("Failed to find playlist with ID {} to add track", id);
                }
            }
            PlaylistAction::RemoveTrack(id, idx) => {
                if let Some(p) = self.playlists.get_playlist_mut(id) {
                    if idx < p.tracks.len() {
                        p.tracks.remove(idx);
                        let path = self.data_dir.join("playlists.json");
                        let _ = self.playlists.save_to_file(&path);
                    }
                }
            }
            PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                info!("Playing track {} from playlist {}", track_idx, playlist_id);
                
                // Find the playlist by ID
                if let Some(playlist) = self.playlists.get_playlist(playlist_id) {
                    // Check if the track index is valid
                    if track_idx < playlist.tracks.len() {
                        // Get the track
                        let track = &playlist.tracks[track_idx];
                        
                        // Get the track path
                        let path = &track.path;
                        
                        // Log the path we're about to play
                        info!("Starting playback of file at path: {}", path);
                        
                        // Trigger playback with explicit error handling
                        match self.player.play(path) {
                            Ok(_) => info!("Successfully started playback"),
                            Err(e) => error!("Failed to play track: {}", e),
                        }
                    } else {
                        error!("Track index {} is out of bounds for playlist {}", track_idx, playlist_id);
                    }
                } else {
                    error!("Could not find playlist with ID {}", playlist_id);
                }
            },
            _ => {} // Handle other cases like None
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