// core/src/library.rs
use serde::{Serialize, Deserialize};
use crate::Track; // Import Track from lib.rs re-export

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
}

#[derive(Clone, Debug)]
pub enum LibraryAction {
    AddScanDirectory(String),
    RemoveScanDirectory(String),
    StartScan,
    ImportFile(String),
    Search(String),
    None,
}