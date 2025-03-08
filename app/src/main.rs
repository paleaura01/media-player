use iced::{Application, Settings, Subscription, Command, Element, Theme};
use core::{PlayerState, LibraryState, PlaylistState, Action};
use core::{Player, PlayerAction, PlaylistAction, LibraryAction};
use std::time::Duration;
use std::path::PathBuf;
use log::{info, error, debug};

// Import UI module directly
mod ui;

// Handle hot reloading in development
// Handle hot reloading in development
#[cfg(debug_assertions)]
mod hot {
    use super::*;
    use hot_lib_reloader::LibReloader;
    use std::path::Path;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use lazy_static::lazy_static;
    
    lazy_static! {
        static ref NEEDS_REFRESH: AtomicBool = AtomicBool::new(false);
    }
    
    pub struct HotUI {
        reloader: RefCell<LibReloader>,
        last_render_time: RefCell<std::time::Instant>,
    }
    
    impl std::fmt::Debug for HotUI {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HotUI").finish()
        }
    }
    
    impl Clone for HotUI {
        fn clone(&self) -> Self {
            Self::new()
        }
    }
    
    impl HotUI {
        pub fn new() -> Self {
            let reloader = match LibReloader::new(
                Path::new("./target/debug"), 
                "player_ui-latest",
                Some(Duration::from_millis(500))
            ) {
                Ok(reloader) => reloader,
                Err(e) => {
                    eprintln!("Failed to initialize hot reloader: {}", e);
                    panic!("Hot reloading initialization failed: {}", e);
                }
            };
            
            info!("Hot reloader initialized successfully");
            Self { 
                reloader: RefCell::new(reloader),
                last_render_time: RefCell::new(std::time::Instant::now()),
            }
        }
        
        pub fn render<'a>(
            &self, 
            player: &'a PlayerState,
            playlists: &'a PlaylistState,
            library: &'a LibraryState
        ) -> Element<'a, Action> {
            // Limit how often we check for updates
            let now = std::time::Instant::now();
            let mut last_time = self.last_render_time.borrow_mut();
            let should_check = now.duration_since(*last_time) > Duration::from_millis(2000);
            
            if should_check {
                *last_time = now;
                
                // Try to update the reloader but handle errors gracefully
                if let Ok(mut reloader) = self.reloader.try_borrow_mut() {
                    match reloader.update() {
                        Ok(true) => {
                            info!("Hot reloading detected changes");
                            NEEDS_REFRESH.store(true, Ordering::SeqCst);
                        },
                        Ok(false) => {},
                        Err(e) => {
                            error!("Hot reloader update error: {}", e);
                            
                            // Try to recover by creating a new reloader
                            drop(reloader);
                            match LibReloader::new(
                                Path::new("./target/debug"),
                                "player_ui-latest",
                                Some(Duration::from_millis(500))
                            ) {
                                Ok(new_reloader) => {
                                    *self.reloader.borrow_mut() = new_reloader;
                                    info!("Created new reloader after error");
                                },
                                Err(e) => {
                                    error!("Failed to create new reloader: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            
            // Check if we need to refresh the UI
            let needs_refresh = NEEDS_REFRESH.swap(false, Ordering::SeqCst);
            
            // Get the render function from the library
            if let Ok(reloader) = self.reloader.try_borrow() {
                // Try to load the render symbol from the library
                match unsafe {
                    reloader.get_symbol::<fn(&'a PlayerState, &'a PlaylistState, &'a LibraryState) -> Element<'a, Action>>(b"render")
                } {
                    Ok(render_fn) => {
                        if needs_refresh {
                            info!("Using updated render function");
                        }
                        render_fn(player, playlists, library)
                    },
                    Err(e) => {
                        // Fall back to the built-in render function
                        error!("Error loading render function: {}", e);
                        ui::render(player, playlists, library)
                    }
                }
            } else {
                // Fall back if we can't borrow the reloader
                ui::render(player, playlists, library)
            }
        }
    }
}


// In release builds, use the UI directly
#[cfg(not(debug_assertions))]
mod hot {
    use super::*;
    
    #[derive(Debug, Clone)]
    pub struct HotUI;
    
    impl HotUI {
        pub fn new() -> Self {
            Self
        }
        
        pub fn render<'a>(
            &self, 
            player: &'a PlayerState,
            playlists: &'a PlaylistState,
            library: &'a LibraryState
        ) -> Element<'a, Action> {
            // In release mode, call render directly
            ui::render(player, playlists, library)
        }
    }
} // End of release mode hot module

use hot::HotUI;

fn main() -> iced::Result {
    // Initialize with appropriate logging level
    std::env::set_var("RUST_LOG", "app=debug,hot_lib_reloader=debug");
    env_logger::init();
    info!("Starting media player application");
    
    MediaPlayer::run(Settings::default())
}

struct MediaPlayer {
    hot_ui: HotUI,
    player: Player,
    player_state: PlayerState,
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
        let data_dir = PathBuf::from("data");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        }
        
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
        
        let player = Player::new();
        let player_state = player.get_state();
        
        info!("Application initialized");
        
        (
            Self {
                hot_ui: HotUI::new(),
                player,
                player_state,
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
                debug!("Processing action: {:?}", action);
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
                            PlayerAction::SetVolume(_vol) => { /* volume control */ },
                            PlayerAction::Seek(_pos) => { /* seek control */ },
                        }
                    },
                    Action::Playlist(action) => {
                        match action {
                            PlaylistAction::Create(name) => {
                                info!("Creating playlist: {}", name);
                                self.playlists.create_playlist(name);
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
                                let path = self.data_dir.join("playlists.bin");
                                if let Err(e) = self.playlists.save_to_file(&path) {
                                    error!("Failed to save playlists: {}", e);
                                }
                            },
                            PlaylistAction::Rename(id, name) => {
                                if let Some(playlist) = self.playlists.get_playlist_mut(id) {
                                    playlist.name = name;
                                    let path = self.data_dir.join("playlists.bin");
                                    if let Err(e) = self.playlists.save_to_file(&path) {
                                        error!("Failed to save playlists: {}", e);
                                    }
                                }
                            },
                            PlaylistAction::AddTrack(id, track) => {
                                if let Some(playlist) = self.playlists.get_playlist_mut(id) {
                                    playlist.tracks.push(track);
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
                                self.library.scanning = true;
                                // Simulate scanning by adding sample tracks
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
                            LibraryAction::Search(_query) => { /* search implementation */ },
                        }
                    },
                }
                // Update player_state after action
                self.player_state = self.player.get_state();
                Command::none()
            },
            Message::Tick => {
                if self.player.get_state().status == core::PlaybackStatus::Playing {
                    self.player.update_progress();
                }
                // Update player_state after tick
                self.player_state = self.player.get_state();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        self.hot_ui.render(&self.player_state, &self.playlists, &self.library)
            .map(Message::Action)
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(100))
            .map(|_| Message::Tick)
    }
} // End of Application implementation