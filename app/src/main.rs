use iced::{Application, Settings, Subscription, Command, Element, Theme};
use core::{PlayerState, LibraryState, PlaylistState, Action};
use core::{Player, PlayerAction, PlaylistAction, LibraryAction};
use hot_lib_reloader::HotReloadableLib;
use std::time::Duration;
use std::path::PathBuf;
use log::{info, error};

// Define the hot-reloadable UI
hot_lib_reloader::hot_functions! {
    /// Hot-reloadable UI implementation
    pub HotUI / "ui" {
        pub fn render(player: &PlayerState, playlists: &PlaylistState, library: &LibraryState) -> Element<Action> in "ui";
    }
}

fn main() -> iced::Result {
    env_logger::init();
    info!("Starting media player application");
    
    // Run the Iced application
    MediaPlayer::run(Settings::default())
}

struct MediaPlayer {
    hot_ui: HotUI,
    player: Player,
    playlists: PlaylistState, 
    library: LibraryState,
    data_dir: PathBuf,
}

#[derive(Debug, Clone)]
enum Message {
    Action(Action),
    Tick,
}

impl Application for MediaPlayer {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Set up data directory
        let data_dir = PathBuf::from("data");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        }
        
        // Try to load playlists
        let playlists_path = data_dir.join("playlists.bin");
        let playlists = if playlists_path.exists() {
            match PlaylistState::load_from_file(&playlists_path) {
                Ok(playlists) => playlists,
                Err(e) => {
                    error!("Failed to load playlists: {}", e);
                    PlaylistState::new()
                }
            }
        } else {
            PlaylistState::new()
        };
        
        (
            Self {
                hot_ui: HotUI::new(),
                player: Player::new(),
                playlists,
                library: LibraryState::new(),
                data_dir,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "Media Player".into()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Action(action) => {
                info!("Processing action: {:?}", action);
                match action {
                    Action::Player(action) => {
                        match action {
                            PlayerAction::Play(path) => {
                                if let Err(e) = self.player.play(&path) {
                                    error!("Failed to play: {}", e);
                                }
                            },
                            PlayerAction::Pause => self.player.pause(),
                            PlayerAction::Resume => self.player.resume(),
                            PlayerAction::Stop => self.player.stop(),
                            PlayerAction::SetVolume(vol) => {
                                // Implementation for volume control
                            },
                            PlayerAction::Seek(pos) => {
                                // Implementation for seeking
                            }
                        }
                    },
                    Action::Playlist(action) => {
                        match action {
                            PlaylistAction::Create(name) => {
                                self.playlists.create_playlist(name);
                                
                                // Save playlists after modification
                                let path = self.data_dir.join("playlists.bin");
                                if let Err(e) = self.playlists.save_to_file(&path) {
                                    error!("Failed to save playlists: {}", e);
                                }
                            },
                            PlaylistAction::Select(id) => {
                                let index = self.playlists.playlists
                                    .iter()
                                    .position(|p| p.id == id);
                                self.playlists.selected = index;
                            },
                            PlaylistAction::Delete(id) => {
                                self.playlists.delete_playlist(id);
                                
                                // Save playlists after modification
                                let path = self.data_dir.join("playlists.bin");
                                if let Err(e) = self.playlists.save_to_file(&path) {
                                    error!("Failed to save playlists: {}", e);
                                }
                            },
                            PlaylistAction::Rename(id, name) => {
                                if let Some(playlist) = self.playlists.get_playlist_mut(id) {
                                    playlist.name = name;
                                    
                                    // Save playlists after modification
                                    let path = self.data_dir.join("playlists.bin");
                                    if let Err(e) = self.playlists.save_to_file(&path) {
                                        error!("Failed to save playlists: {}", e);
                                    }
                                }
                            },
                            PlaylistAction::AddTrack(id, track) => {
                                if let Some(playlist) = self.playlists.get_playlist_mut(id) {
                                    playlist.tracks.push(track);
                                    
                                    // Save playlists after modification
                                    let path = self.data_dir.join("playlists.bin");
                                    if let Err(e) = self.playlists.save_to_file(&path) {
                                        error!("Failed to save playlists: {}", e);
                                    }
                                }
                            },
                            PlaylistAction::RemoveTrack(id, index) => {
                                if let Some(playlist) = self.playlists.get_playlist_mut(id) {
                                    if index < playlist.tracks.len() {
                                        playlist.tracks.remove(index);
                                        
                                        // Save playlists after modification
                                        let path = self.data_dir.join("playlists.bin");
                                        if let Err(e) = self.playlists.save_to_file(&path) {
                                            error!("Failed to save playlists: {}", e);
                                        }
                                    }
                                }
                            },
                        }
                    },
                    Action::Library(action) => {
                        match action {
                            LibraryAction::AddScanDirectory(dir) => {
                                self.library.scan_dirs.push(dir);
                            },
                            LibraryAction::RemoveScanDirectory(dir) => {
                                self.library.scan_dirs.retain(|d| d != &dir);
                            },
                            LibraryAction::StartScan => {
                                // Set scanning flag
                                self.library.scanning = true;
                                
                                // In a real implementation, you would:
                                // 1. Spawn a background task to scan directories
                                // 2. Update the library with found tracks
                                // 3. Clear the scanning flag when done
                                
                                // For now, we'll just simulate finding some tracks
                                self.library.tracks.push(core::Track {
                                    path: "sample1.mp3".to_string(),
                                    title: Some("Sample Track 1".to_string()),
                                    artist: Some("Artist 1".to_string()),
                                    album: Some("Album 1".to_string()),
                                });
                                
                                self.library.tracks.push(core::Track {
                                    path: "sample2.mp3".to_string(),
                                    title: Some("Sample Track 2".to_string()),
                                    artist: Some("Artist 2".to_string()),
                                    album: Some("Album 2".to_string()),
                                });
                                
                                self.library.scanning = false;
                            },
                            LibraryAction::ImportFile(path) => {
                                let filename = std::path::Path::new(&path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(&path)
                                    .to_string();
                                
                                self.library.tracks.push(core::Track {
                                    path,
                                    title: Some(filename),
                                    artist: None,
                                    album: None,
                                });
                            },
                            LibraryAction::Search(_query) => {
                                // Implementation for search
                            },
                        }
                    },
                }
                Command::none()
            },
            Message::Tick => {
                // Update player progress if playing
                if let Some(current_track) = &self.player.get_state().current_track {
                    // In a real implementation, you would get actual progress from the player
                    // For now we'll just simulate progress
                    if self.player.get_state().status == core::PlaybackStatus::Playing {
                        self.player.update_progress();
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Get references to the states
        let player_state = self.player.get_state();
        
        // Render UI through hot-reloadable library
        self.hot_ui.render(player_state, &self.playlists, &self.library)
            .map(Message::Action)
    }

    fn subscription(&self) -> Subscription<Message> {
        // Create a subscription for periodic refreshes
        iced::time::every(Duration::from_millis(100))
            .map(|_| Message::Tick)
    }
}