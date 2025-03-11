use iced::{Element, window, Subscription, Point};
use core::{
    Action, LibraryAction, LibraryState, Player, PlayerAction, PlayerState,
    PlaylistAction, PlaylistState, Track,
};
use log::{debug, error, info};
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};

mod ui;

// -------------------- Window position storage --------------------
#[derive(Debug, Serialize, Deserialize, Default)]
struct WindowPosition {
    x: Option<i32>,
    y: Option<i32>,
}

fn save_window_position(x: i32, y: i32) -> std::io::Result<()> {
    let pos = WindowPosition {
        x: Some(x),
        y: Some(y),
    };
    
    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    
    let json = serde_json::to_string_pretty(&pos)?;
    fs::write("data/window_position.json", json)
}

fn load_window_position() -> WindowPosition {
    let path = PathBuf::from("data/window_position.json");
    if path.exists() {
        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(pos) = serde_json::from_str::<WindowPosition>(&data) {
                return pos;
            }
        }
    }
    WindowPosition::default()
}

// -------------------- Main App State --------------------
pub struct MediaPlayer {
    player: Player,
    player_state: PlayerState,
    playlists: PlaylistState,
    library: LibraryState,
    data_dir: PathBuf,
    window_position: WindowPosition,
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
        let window_position = load_window_position();

        info!("MediaPlayer default state created");

        Self {
            player,
            player_state,
            playlists,
            library: LibraryState::new(),
            data_dir,
            window_position,
        }
    }
}

// -------------------- Iced Messages --------------------
#[derive(Debug, Clone)]
enum Message {
    Action(Action),
    WindowMoved(i32, i32),
    Ignore,
}

// -------------------- update --------------------
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
        Message::WindowMoved(x, y) => {
            // Save new window coords
            let _ = save_window_position(x, y);
            state.window_position.x = Some(x);
            state.window_position.y = Some(y);
        }
        Message::Ignore => {}
    }
    iced::Task::none()
}

// -------------------- view --------------------
fn view(state: &MediaPlayer) -> Element<Message> {
    ui::render(&state.player_state, &state.playlists, &state.library)
        .map(Message::Action)
}

// -------------------- subscription --------------------
fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    // in iced 0.13.x, `window::events()` emits (window::Id, window::Event)
    window::events().map(|(_id, event)| match event {
        // `Moved` is a single-field tuple variant: Moved(Point)
        window::Event::Moved(point) => {
            Message::WindowMoved(point.x as i32, point.y as i32)
        }
        _ => Message::Ignore,
    })
}

// -------------------- main --------------------
fn main() -> iced::Result {
    std::env::set_var("RUST_LOG", "app=debug");
    env_logger::init();
    info!("Starting media player application.");

    // Load last window position
    let window_pos = load_window_position();
    let x = window_pos.x.unwrap_or(100) as f32;
    let y = window_pos.y.unwrap_or(100) as f32;

    // Build application with .window(...) (no title field in window::Settings)
    iced::application("Media Player", update, view)
        .window(window::Settings {
            position: window::Position::Specific(Point::new(x, y)),
            ..window::Settings::default()
        })
        .subscription(subscription)
        .run()
}
