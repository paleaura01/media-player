use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Track {
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

impl Track {
    pub fn new(path: String) -> Self {
        let title = Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string());
            
        Self {
            path,
            title,
            artist: None,
            album: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Playlist {
    pub id: u32,
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaylistState {
    pub playlists: Vec<Playlist>,
    pub selected: Option<usize>,
    next_id: u32,
}

impl PlaylistState {
    pub fn new() -> Self {
        Self {
            playlists: Vec::new(),
            selected: None,
            next_id: 1,
        }
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if path.metadata()?.len() == 0 {
            // Empty file, return default state
            return Ok(Self::new());
        }
        
        let data = fs::read(path)?;
        let state = bincode::deserialize(&data)?;
        Ok(state)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let data = bincode::serialize(self)?;
        fs::write(path, data)?;
        Ok(())
    }
    
    pub fn create_playlist(&mut self, name: String) -> Playlist {
        let id = self.next_id;
        self.next_id += 1;
        
        let playlist = Playlist {
            id,
            name,
            tracks: Vec::new(),
        };
        
        self.playlists.push(playlist.clone());
        playlist
    }
    
    pub fn delete_playlist(&mut self, id: u32) {
        if let Some(pos) = self.playlists.iter().position(|p| p.id == id) {
            self.playlists.remove(pos);
            
            // Update selected if needed
            if let Some(selected) = self.selected {
                if selected == pos {
                    self.selected = None;
                } else if selected > pos {
                    self.selected = Some(selected - 1);
                }
            }
        }
    }
    
    pub fn get_playlist_mut(&mut self, id: u32) -> Option<&mut Playlist> {
        self.playlists.iter_mut().find(|p| p.id == id)
    }
    
    pub fn get_playlist(&self, id: u32) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.id == id)
    }
}

#[derive(Clone, Debug)]
pub enum PlaylistAction {
    Create(String),
    Delete(u32),
    Select(u32),
    Rename(u32, String),
    AddTrack(u32, Track),
    RemoveTrack(u32, usize),
}