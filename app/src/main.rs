// app/src/main.rs
use iced::{Element};
use core::{
    Action, LibraryAction, LibraryState, Player, PlayerAction, PlayerState,
    PlaylistAction, PlaylistState, Track,
};
use log::{debug, error, info};
use std::path::PathBuf;

// Import your UI rendering module as normal
mod ui;

// -------------------- Main App State --------------------
pub struct MediaPlayer {
    player: Player,
    player_state: PlayerState,
    playlists: PlaylistState,
    library: LibraryState,
    data_dir: PathBuf,
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
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Action(Action),
}

// Update function that will be passed to iced::run
fn update(state: &mut MediaPlayer, message: Message) -> iced::Task<Message> {
    match message {
        Message::Action(action) => {
            debug!("Handling Action: {:?}", action);
            match action {
                Action::Player(act) => match act {
                    PlayerAction::Play(path) => {
                        if let Err(e) = state.player.play(&path) {
                            error!("Failed to play: {}", e);
                        }
                    }
                    PlayerAction::Pause => state.player.pause(),
                    PlayerAction::Resume => state.player.resume(),
                    PlayerAction::Stop => state.player.stop(),
                    PlayerAction::SetVolume(_v) => {}
                    PlayerAction::Seek(_pos) => {}
                },
                Action::Playlist(act) => match act {
                    PlaylistAction::Create(name) => {
                        info!("Creating playlist: {}", name);
                        state.playlists.create_playlist(name);
                        let path = state.data_dir.join("playlists.json");
                        let _ = state.playlists.save_to_file(&path);
                    }
                    PlaylistAction::Select(id) => {
                        let idx = state.playlists.playlists.iter().position(|p| p.id == id);
                        state.playlists.selected = idx;
                    }
                    PlaylistAction::Delete(id) => {
                        state.playlists.delete_playlist(id);
                        let path = state.data_dir.join("playlists.json");
                        let _ = state.playlists.save_to_file(&path);
                    }
                    PlaylistAction::Rename(id, nm) => {
                        if let Some(p) = state.playlists.get_playlist_mut(id) {
                            p.name = nm;
                            let path = state.data_dir.join("playlists.json");
                            let _ = state.playlists.save_to_file(&path);
                        }
                    }
                    PlaylistAction::AddTrack(id, track) => {
                        if let Some(p) = state.playlists.get_playlist_mut(id) {
                            p.tracks.push(track);
                            let path = state.data_dir.join("playlists.json");
                            let _ = state.playlists.save_to_file(&path);
                        }
                    }
                    PlaylistAction::RemoveTrack(id, idx) => {
                        if let Some(p) = state.playlists.get_playlist_mut(id) {
                            if idx < p.tracks.len() {
                                p.tracks.remove(idx);
                                let path = state.data_dir.join("playlists.json");
                                let _ = state.playlists.save_to_file(&path);
                            }
                        }
                    }
                    _ => {}
                },
                Action::Library(act) => match act {
                    LibraryAction::AddScanDirectory(dir) => {
                        state.library.scan_dirs.push(dir);
                    }
                    LibraryAction::RemoveScanDirectory(dir) => {
                        state.library.scan_dirs.retain(|d| d != &dir);
                    }
                    LibraryAction::StartScan => {
                        state.library.scanning = true;
                        state.library.tracks.push(Track {
                            path: "sample1.mp3".to_string(),
                            title: Some("Sample Track 1".to_string()),
                            artist: Some("Artist 1".to_string()),
                            album: Some("Album 1".to_string()),
                        });
                        state.library.tracks.push(Track {
                            path: "sample2.mp3".to_string(),
                            title: Some("Sample Track 2".to_string()),
                            artist: Some("Artist 2".to_string()),
                            album: Some("Album 2".to_string()),
                        });
                        state.library.scanning = false;
                    }
                    LibraryAction::ImportFile(path) => {
                        let filename = std::path::Path::new(&path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&path)
                            .to_string();
                        state.library.tracks.push(Track {
                            path,
                            title: Some(filename),
                            artist: None,
                            album: None,
                        });
                    }
                    LibraryAction::Search(_q) => {}
                },
            }
            state.player_state = state.player.get_state();
        }
    }
    
    // Return an empty task since we're not doing any async work
    iced::Task::none()
}

// View function that will be passed to iced::run - Use the standard UI rendering
fn view(state: &MediaPlayer) -> Element<Message> {
    // Call the UI render function directly, no hot reloading for now
    let ui_element = ui::render(&state.player_state, &state.playlists, &state.library);
    
    // Map the UI element to our Message type
    ui_element.0.map(Message::Action)
}

fn main() -> iced::Result {
    std::env::set_var("RUST_LOG", "app=debug");
    env_logger::init();
    info!("Starting media player application.");
    
    // Use the correct pattern with 3 arguments as per documentation
    iced::run("Media Player", update, view)
}