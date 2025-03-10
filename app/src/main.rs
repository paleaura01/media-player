// app/src/main.rs
use iced::{Element};
use core::{
    Action, LibraryAction, LibraryState, Player, PlayerAction, PlayerState,
    PlaylistAction, PlaylistState, Track,
};
use log::{debug, error, info};
use std::path::PathBuf;

// Import your UI rendering module
mod ui;

// --- Hot Reloading for debug builds ---
#[cfg(debug_assertions)]
mod hot {
    use super::*;
    use hot_lib_reloader::LibReloader;
    use std::cell::RefCell;
    use std::path::Path;
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

    // Manually implement Clone since LibReloader is not Clone.
    impl Clone for HotUI {
        fn clone(&self) -> Self {
            Self::new()
        }
    }

    // Manually implement Debug since LibReloader doesn't implement Debug
    impl std::fmt::Debug for HotUI {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("HotUI")
                .field("last_update", &"<reloader>")
                .finish()
        }
    }

    impl HotUI {
        pub fn new() -> Self {
            let reloader = match LibReloader::new(
                Path::new("./target/debug"),
                "player_ui-latest",
                Some(Duration::from_millis(500)),
            ) {
                Ok(r) => r,
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
            library: &'a LibraryState,
        ) -> Element<'a, Action> {
            // Check for updates every 2 seconds.
            let now = std::time::Instant::now();
            let mut last = self.last_render_time.borrow_mut();
            if now.duration_since(*last) > Duration::from_millis(2000) {
                *last = now;
                if let Ok(mut reloader) = self.reloader.try_borrow_mut() {
                    match reloader.update() {
                        Ok(true) => {
                            info!("Detected changes in UI library");
                            NEEDS_REFRESH.store(true, Ordering::SeqCst);
                        }
                        Ok(false) => {}
                        Err(e) => {
                            error!("Hot reloader update error: {}", e);
                            drop(reloader);
                            if let Ok(new_r) = LibReloader::new(
                                Path::new("./target/debug"),
                                "player_ui-latest",
                                Some(Duration::from_millis(500)),
                            ) {
                                *self.reloader.borrow_mut() = new_r;
                                info!("Reinitialized reloader after error");
                            } else {
                                error!("Reinit reloader failed");
                            }
                        }
                    }
                }
            }

            if let Ok(reloader) = self.reloader.try_borrow() {
                match unsafe {
                    reloader.get_symbol::<fn(
                        &'a PlayerState,
                        &'a PlaylistState,
                        &'a LibraryState
                    ) -> Element<'a, Action>>(b"render")
                } {
                    Ok(render_fn) => render_fn(player, playlists, library),
                    Err(e) => {
                        error!("Failed to load hot render fn: {}", e);
                        ui::render(player, playlists, library)
                    }
                }
            } else {
                ui::render(player, playlists, library)
            }
        }
    }
}

#[cfg(not(debug_assertions))]
mod hot {
    use super::*;
    
    #[derive(Clone, Debug)]
    pub struct HotUI;

    impl HotUI {
        pub fn new() -> Self {
            Self
        }

        pub fn render<'a>(
            &self,
            player: &'a PlayerState,
            playlists: &'a PlaylistState,
            library: &'a LibraryState,
        ) -> Element<'a, Action> {
            ui::render(player, playlists, library)
        }
    }
}

use hot::HotUI;

// -------------------- Main App State --------------------
pub struct MediaPlayer {
    hot_ui: HotUI,
    player: Player,
    player_state: PlayerState,
    playlists: PlaylistState,
    library: LibraryState,
    data_dir: PathBuf,
}

// Manually implement Debug since Player doesn't implement Debug
impl std::fmt::Debug for MediaPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaPlayer")
            .field("hot_ui", &self.hot_ui)
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
            hot_ui: HotUI::new(),
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

// View function that will be passed to iced::run
fn view(state: &MediaPlayer) -> Element<Message> {
    state.hot_ui
        .render(&state.player_state, &state.playlists, &state.library)
        .map(Message::Action)
}

fn main() -> iced::Result {
    std::env::set_var("RUST_LOG", "app=debug,hot_lib_reloader=debug");
    env_logger::init();
    info!("Starting media player application.");
    
    // Use the correct pattern with 3 arguments as per documentation
    iced::run("Media Player", update, view)
}