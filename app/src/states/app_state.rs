// app/src/states/app_state.rs
// This file handles the core application state and actions

use std::path::PathBuf;
use std::time::{Instant, Duration};
use log::{debug, error, info};
use core::{Action, PlayerAction, PlaylistAction, LibraryAction, Track, Player, PlayerState, PlaybackStatus, PlaylistState, LibraryState};
use crate::states::playlist_state::PlaylistViewState;
use rand::Rng; // For picking random track if shuffle is on
use anyhow::Result;
use std::fs;

pub struct MediaPlayer {
    pub player: Player,
    pub player_state: PlayerState,
    pub playlists: PlaylistState,
    pub library: LibraryState,
    pub data_dir: PathBuf,
    pub playlist_view_state: PlaylistViewState,
    pub status_message: Option<String>,              // For displaying status messages
    pub status_message_time: Option<Instant>,        // When the message was set
    pub status_message_duration: Option<Duration>,   // How long to show it
    pub is_batch_processing: bool,                   // Track when batch processing is active
}

impl std::fmt::Debug for MediaPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaPlayer")
            .field("player", &"<player>")
            .field("player_state", &self.player_state)
            .field("playlists", &self.playlists)
            .field("library", &self.library)
            .field("data_dir", &self.data_dir)
            .field("status_message", &self.status_message)
            .field("is_batch_processing", &self.is_batch_processing)
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

        // Initialize the player with FFmpeg
        let mut player = Player::new();
        // Configure network buffer settings
        player.configure_network(8 * 1024 * 1024, 5); // 8MB buffer, 5 second pre-buffering
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
            status_message: None,
            status_message_time: None,
            status_message_duration: None,
            is_batch_processing: false,
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
    
    // Helper method to get a smart-shuffled track index
    fn get_smart_shuffled_track_index(&self, playlist_id: u32) -> Option<usize> {
        if let Some(playlist) = self.playlists.get_playlist(playlist_id) {
            if playlist.tracks.is_empty() {
                return None;
            }
            
            // Find the minimum play count in the playlist
            let min_play_count = playlist.tracks
                .iter()
                .map(|track| track.play_count)
                .min()
                .unwrap_or(0);
            
            // Filter tracks that have this minimum play count
            let candidate_tracks: Vec<usize> = playlist.tracks
                .iter()
                .enumerate()
                .filter(|(_, track)| track.play_count == min_play_count)
                .map(|(i, _)| i)
                .collect();
            
            if !candidate_tracks.is_empty() {
                // Pick a random track from the filtered list
                let random_idx = rand::thread_rng().gen_range(0..candidate_tracks.len());
                return Some(candidate_tracks[random_idx]);
            }
        }
        None
    }
    
    fn handle_player_action(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Play(path) => {
                info!("Attempting to play file: {}", path);
                
                // For network paths, show a status message but don't do special buffering
                if path.starts_with("\\\\") || path.contains("://") {
                    self.status_message = Some(format!("Direct streaming: {}", 
                        std::path::Path::new(&path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")));
                    self.status_message_time = Some(Instant::now());
                    self.status_message_duration = Some(Duration::from_secs(3));
                }
                
                // Play the file directly - no buffering needed
                if let Err(e) = self.player.play(&path) {
                    error!("Failed to play: {}", e);
                    self.status_message = Some(format!("Error: {}", e));
                    self.status_message_time = Some(Instant::now());
                    self.status_message_duration = Some(Duration::from_secs(3));
                } else {
                    info!("Started playback successfully");
                }
            }
            PlayerAction::Pause => self.player.pause(),
            PlayerAction::Resume => {
                // Check if we're already playing or paused
                if self.player_state.status == PlaybackStatus::Paused {
                    // Original resume logic for a paused track
                    info!("Resuming playback");
                    self.player.resume();
                } else if self.player_state.status == PlaybackStatus::Stopped {
                    // New logic to start playing a track when nothing is playing
                    info!("Starting playback from Now Playing list");
                    
                    // Check if we have a selected playlist
                    if let Some(idx) = self.playlists.selected {
                        if idx < self.playlists.playlists.len() {
                            let playlist = &self.playlists.playlists[idx];
                            
                            if !playlist.tracks.is_empty() {
                                let track_idx = if self.player_state.shuffle_enabled {
                                    // Get a smart-shuffled track index
                                    if let Some(idx) = self.get_smart_shuffled_track_index(playlist.id) {
                                        idx
                                    } else {
                                        0 // Fallback to first track
                                    }
                                } else {
                                    // No shuffle, just start with the first track
                                    0
                                };
                                
                                let track = &playlist.tracks[track_idx];
                                info!("Auto-playing track: {}", track.title.as_ref().unwrap_or(&track.path));
                                
                                // Play the selected track
                                self.handle_action(core::Action::Playlist(
                                    core::PlaylistAction::PlayTrack(playlist.id, track_idx)
                                ));
                            }
                        }
                    }
                }
            },
            PlayerAction::Stop => self.player.stop(),
            PlayerAction::SetVolume(v) => self.player.set_volume(v),
            PlayerAction::Seek(pos) => self.player.seek(pos),
            PlayerAction::SkipForward(seconds) => {
                if let Some(current) = self.player_state.position {
                    if let Some(duration) = self.player_state.duration {
                        // Calculate relative position
                        let total_secs = duration.as_secs_f32();
                        let current_secs = current.as_secs_f32();
                        let new_secs = (current_secs + seconds).min(total_secs);
                        let new_pos = new_secs / total_secs;
                        info!("Skipping forward {} seconds to position {:.2}", seconds, new_pos);
                        self.player.seek(new_pos);
                    }
                }
            },
            PlayerAction::SkipBackward(seconds) => {
                if let Some(current) = self.player_state.position {
                    if let Some(duration) = self.player_state.duration {
                        // Calculate relative position
                        let total_secs = duration.as_secs_f32();
                        let current_secs = current.as_secs_f32();
                        let new_secs = (current_secs - seconds).max(0.0);
                        let new_pos = new_secs / total_secs;
                        info!("Skipping backward {} seconds to position {:.2}", seconds, new_pos);
                        self.player.seek(new_pos);
                    }
                }
            },
            PlayerAction::Shuffle => {
                // Toggle shuffle mode
                self.player_state.shuffle_enabled = !self.player_state.shuffle_enabled;
                if self.player_state.shuffle_enabled {
                    info!("Shuffle enabled");
                } else {
                    info!("Shuffle disabled");
                }
            },
            PlayerAction::NextTrack => {
                // Code for next track with smart shuffle consideration
                info!("Next track button pressed");
                
                if let Some(idx) = self.playlists.selected {
                    if idx < self.playlists.playlists.len() {
                        let playlist = &self.playlists.playlists[idx];
                        
                        if self.player_state.shuffle_enabled {
                            // If shuffle is enabled, use smart shuffle logic
                            if !playlist.tracks.is_empty() {
                                if let Some(track_idx) = self.get_smart_shuffled_track_index(playlist.id) {
                                    let track = &playlist.tracks[track_idx];
                                    info!("Playing least-played track: {} (play count: {})", 
                                          track.title.as_deref().unwrap_or(&track.path), track.play_count);
                                    
                                    self.handle_action(core::Action::Playlist(
                                        core::PlaylistAction::PlayTrack(playlist.id, track_idx)
                                    ));
                                }
                            }
                        } else {
                            // Original sequential next track logic (unchanged)
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
                                // No track playing, start with first
                                self.handle_action(core::Action::Playlist(
                                    core::PlaylistAction::PlayTrack(playlist.id, 0)
                                ));
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
                        
                        if self.player_state.shuffle_enabled {
                            // Use smart shuffle for previous as well
                            if !playlist.tracks.is_empty() {
                                if let Some(track_idx) = self.get_smart_shuffled_track_index(playlist.id) {
                                    let track = &playlist.tracks[track_idx];
                                    info!("Playing least-played track: {} (play count: {})", 
                                          track.title.as_deref().unwrap_or(&track.path), track.play_count);
                                    
                                    self.handle_action(core::Action::Playlist(
                                        core::PlaylistAction::PlayTrack(playlist.id, track_idx)
                                    ));
                                }
                            }
                        } else {
                            // Original sequential previous track logic
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
                }
            },
        }
    }
    
    fn handle_playlist_action(&mut self, action: PlaylistAction) {
        match action {
            PlaylistAction::Create(name) => {
                self.playlists.create_playlist(name);
                // Save after creating playlist
                let _ = self.save_playlists();
            },
            PlaylistAction::Delete(id) => {
                self.playlists.delete_playlist(id);
                // Save after deleting playlist
                let _ = self.save_playlists();
            },
            PlaylistAction::Select(id) => {
                if let Some(pos) = self.playlists.playlists.iter().position(|p| p.id == id) {
                    self.playlists.selected = Some(pos);
                    // No need to save for selection changes
                }
            },
            PlaylistAction::Rename(id, new_name) => {
                let renamed = self.playlists.rename_playlist(id, new_name);
                if renamed {
                    // Save after renaming playlist
                    let _ = self.save_playlists();
                }
            },
            PlaylistAction::AddTrack(playlist_id, track) => {
                if let Some(playlist) = self.playlists.get_playlist_mut(playlist_id) {
                    playlist.tracks.push(track);
                    // Save after adding track
                    let _ = self.save_playlists();
                }
            },
            PlaylistAction::RemoveTrack(playlist_id, index) => {
                if let Some(playlist) = self.playlists.get_playlist_mut(playlist_id) {
                    if index < playlist.tracks.len() {
                        playlist.tracks.remove(index);
                        // Save after removing track
                        let _ = self.save_playlists();
                    }
                }
            },
            PlaylistAction::BatchAddTracks(playlist_id, tracks) => {
                if let Some(playlist) = self.playlists.get_playlist_mut(playlist_id) {
                    info!("Adding batch of {} tracks to playlist {}", tracks.len(), playlist_id);
                    playlist.tracks.extend(tracks);
                    
                    // Only save if we're not in the middle of a large batch operation
                    if !self.is_batch_processing {
                        if let Err(e) = self.save_playlists() {
                            error!("Failed to save playlist after batch add: {}", e);
                        } else {
                            info!("Successfully saved playlist after batch add");
                        }
                    }
                }
            },
            PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                if let Some(playlist) = self.playlists.get_playlist(playlist_id) {
                    if track_idx < playlist.tracks.len() {
                        let track = &playlist.tracks[track_idx];
                        
                        // Check if it's a network path before playing
                        if track.path.starts_with("\\\\") || track.path.contains("://") {
                            self.status_message = Some(format!("Loading network track: {}", 
                                track.title.as_deref().unwrap_or("Unknown")));
                            self.status_message_time = Some(Instant::now());
                            self.status_message_duration = Some(Duration::from_secs(5));
                        }
                        
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
                // Scan logic would go here
                // For demonstration, just add sample tracks
                self.library.tracks.push(Track {
                    path: "sample1.mp3".to_string(),
                    title: Some("Sample Track 1".to_string()),
                    artist: Some("Artist 1".to_string()),
                    album: Some("Album 1".to_string()),
                    play_count: 0,
                });
                self.library.tracks.push(Track {
                    path: "sample2.mp3".to_string(),
                    title: Some("Sample Track 2".to_string()),
                    artist: Some("Artist 2".to_string()),
                    album: Some("Album 2".to_string()),
                    play_count: 0,
                });
                // Add a sample Opus file to show support
                self.library.tracks.push(Track {
                    path: "sample.opus".to_string(),
                    title: Some("Sample Opus Track".to_string()),
                    artist: Some("Artist 3".to_string()),
                    album: Some("Album 3".to_string()),
                    play_count: 0,
                });
                self.library.scanning = false;
            }
            LibraryAction::ImportFile(path) => {
                let file_path = std::path::Path::new(&path);
                let filename = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
                
                // Check if file is a valid audio file using FFmpeg
                if core::audio::decoder::is_supported_audio_format(&path) {
                    // Log before moving the path
                    info!("Imported audio file: {}", path);
                    
                    // Now move the path into the Track
                    self.library.tracks.push(Track {
                        path,
                        title: Some(filename),
                        artist: None,
                        album: None,
                        play_count: 0,
                    });
                } else {
                    info!("Skipped unsupported file format: {}", path);
                }
            }
            LibraryAction::Search(_q) => {}
            LibraryAction::None => {}
        }
    }
    
    // Add track completion handling
    pub fn check_for_completed_tracks(&mut self) {
        // If the player signals that a track was completed
        if self.player.track_completed_signal {
            // Reset the signal
            self.player.track_completed_signal = false;
            
            // Get the currently playing track path
            if let Some(track_path) = &self.player_state.current_track {
                // Find and update the track's play count in the playlists
                for playlist in &mut self.playlists.playlists {
                    for track in &mut playlist.tracks {
                        if &track.path == track_path {
                            track.play_count += 1;
                            info!("Updated play count for '{}' to {}", 
                                  track.title.as_ref().unwrap_or(&track.path), 
                                  track.play_count);
                            break;
                        }
                    }
                }
                
                // Save updated play counts to disk
                if let Err(e) = self.save_playlists() {
                    error!("Failed to save play count: {}", e);
                }
                
                // Auto-play the next track
                self.handle_action(core::Action::Player(core::PlayerAction::NextTrack));
            }
        }
    }

    // Improved save_playlists implementation with atomic file operations
    pub fn save_playlists(&self) -> Result<(), anyhow::Error> {
        let path = self.data_dir.join("playlists.json");
        info!("Saving playlists to {}", path.display());
        
        // Create the JSON string first
        let json_data = serde_json::to_string_pretty(&self.playlists)?;
        
        // First write to a temporary file
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, &json_data)?;
        
        // Then rename the temporary file to the actual file, which is an atomic operation
        fs::rename(&temp_path, &path)?;
        
        info!("Successfully saved playlists");
        Ok(())
    }
}