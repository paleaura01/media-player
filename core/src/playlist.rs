use std::path::Path;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::fs;
use std::time::Instant;

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
        Self { path, title, artist: None, album: None }
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
    #[serde(skip)]
    pub editing_id: Option<u32>,  // Track which playlist is being edited
    #[serde(skip)]
    pub editing_text: String,     // Track the current editing text
    #[serde(skip)]
    pub last_clicked_id: Option<u32>, // Track last clicked playlist for double-click detection
    #[serde(skip)]
    pub last_clicked_time: Option<Instant>, // Track timestamp of last click
    #[serde(skip)]
    pub showing_create_popup: bool, // Flag to show the create playlist popup
    #[serde(skip)]
    pub new_playlist_name: String, // Text field for new playlist name
    #[serde(skip)]
    pub deleting_playlist_id: Option<u32>, // Track which playlist is pending deletion
    next_id: u32,
}

impl PlaylistState {
    pub fn new() -> Self {
        Self { 
            playlists: Vec::new(), 
            selected: None, 
            editing_id: None,
            editing_text: String::new(),
            last_clicked_id: None,
            last_clicked_time: None,
            showing_create_popup: false,
            new_playlist_name: "New Playlist".to_string(),
            deleting_playlist_id: None,
            next_id: 1 
        }
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        // Check if file exists and has content
        if !path.exists() || path.metadata()?.len() == 0 {
            return Ok(Self::new());
        }
        
        // Read file as string
        let content = fs::read_to_string(path)?;
        
        // Parse JSON
        let state: Self = serde_json::from_str(&content)?;
        Ok(state)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Serialize to JSON
        let json = serde_json::to_string_pretty(self)?;
        
        // Write to file
        fs::write(path, json)?;
        Ok(())
    }
    
    pub fn create_playlist(&mut self, name: String) -> Playlist {
        let id = self.next_id;
        self.next_id += 1;
        let playlist = Playlist { id, name, tracks: Vec::new() };
        self.playlists.push(playlist.clone());
        
        // Select the newly created playlist
        self.selected = Some(self.playlists.len() - 1);
        
        playlist
    }
    
    pub fn delete_playlist(&mut self, id: u32) {
        if let Some(pos) = self.playlists.iter().position(|p| p.id == id) {
            self.playlists.remove(pos);
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
    
    pub fn rename_playlist(&mut self, id: u32, new_name: String) -> bool {
        if let Some(playlist) = self.get_playlist_mut(id) {
            playlist.name = new_name;
            true
        } else {
            false
        }
    }
    
    pub fn add_track_to_selected(&mut self, track: Track) -> bool {
        if let Some(idx) = self.selected {
            if idx < self.playlists.len() {
                let playlist_id = self.playlists[idx].id;
                if let Some(playlist) = self.get_playlist_mut(playlist_id) {
                    playlist.tracks.push(track);
                    return true;
                }
            }
        }
        false
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
    Click(u32),                // For playlist click (to implement double-click)
    StartEditing(u32),         // Start editing a playlist name
    UpdateEditingText(String), // Update the editing text as the user types
    SaveEdit,                  // Save the current edit
    CancelEdit,                // Cancel editing
    ClickOutside,              // Handle clicking outside the editing area
    ShowCreatePopup,           // Show the create playlist popup
    HideCreatePopup,           // Hide the create playlist popup
    UpdateNewPlaylistName(String), // Update the new playlist name text field
    ShowDeleteConfirmation(u32), // Show the delete confirmation popup
    CancelDelete,              // Cancel the delete operation
    ConfirmDelete,             // Confirm the delete operation
}