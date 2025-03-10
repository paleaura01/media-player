// app/src/app_state.rs
use std::path::PathBuf;
use log::{debug, error, info};
use core::{
    Action, LibraryAction, LibraryState, Player, PlayerAction, PlayerState,
    PlaylistAction, PlaylistState, Track,
};

pub struct MediaPlayer {
    pub player: Player,
    pub player_state: PlayerState,
    pub playlists: PlaylistState,
    pub library: LibraryState,
    pub data_dir: PathBuf,
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
                if let Err(e) = self.player.play(&path) {
                    error!("Failed to play: {}", e);
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
                let idx = self.playlists.playlists.iter().position(|p| p.id == id);
                self.playlists.selected = idx;
            }
            PlaylistAction::Delete(id) => {
                self.playlists.delete_playlist(id);
                let path = self.data_dir.join("playlists.json");
                let _ = self.playlists.save_to_file(&path);
            }
            PlaylistAction::Rename(id, nm) => {
                if let Some(p) = self.playlists.get_playlist_mut(id) {
                    p.name = nm;
                    let path = self.data_dir.join("playlists.json");
                    let _ = self.playlists.save_to_file(&path);
                }
            }
            PlaylistAction::AddTrack(id, track) => {
                if let Some(p) = self.playlists.get_playlist_mut(id) {
                    p.tracks.push(track);
                    let path = self.data_dir.join("playlists.json");
                    let _ = self.playlists.save_to_file(&path);
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
