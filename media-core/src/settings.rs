use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Result as IoResult;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub window_position: Option<(i32, i32)>,
    pub last_file_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            window_position: None,
            last_file_path: None,
        }
    }
}

impl AppSettings {
    pub fn config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "media-player", "rust-media-player") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir).unwrap_or_default();
            config_dir.join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match File::open(&path) {
            Ok(file) => {
                serde_json::from_reader(file).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> IoResult<()> {
        let path = Self::config_path();
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}