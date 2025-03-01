// src/playlist.rs
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub file_path: PathBuf,
    // You can extend Track with more metadata if needed (title, duration, etc.)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Playlist {
    pub name: Option<String>,
    pub tracks: Vec<Track>,
}

impl Playlist {
    /// Load a playlist from a JSON file.
    pub fn load_from_file(path: &str) -> std::io::Result<Playlist> {
        let file = File::open(path)?;
        let playlist: Playlist = serde_json::from_reader(file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(playlist)
    }

    /// Save the playlist to a JSON file (pretty-printed).
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new().create(true).write(true).truncate(true).open(path)?;
        serde_json::to_writer_pretty(file, self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
