use serde::{Serialize, Deserialize};
use super::playlist::Track;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryState {
    pub tracks: Vec<Track>,
    pub scan_dirs: Vec<String>,
    pub scanning: bool,
}

impl LibraryState {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            scan_dirs: vec!["./music".to_string()],
            scanning: false,
        }
    }
    
    pub fn scan_directory(&mut self, dir: &str) -> anyhow::Result<()> {
        self.scanning = true;
        let path = std::path::Path::new(dir);
        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if ["mp3", "wav", "flac", "ogg"].contains(&ext_str) {
                                let path_str = path.to_string_lossy().to_string();
                                let title = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s.to_string());
                                self.tracks.push(Track {
                                    path: path_str,
                                    title,
                                    artist: None,
                                    album: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        self.scanning = false;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum LibraryAction {
    AddScanDirectory(String),
    RemoveScanDirectory(String),
    StartScan,
    ImportFile(String),
    Search(String),
}
