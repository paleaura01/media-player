use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WindowPosition {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

pub fn save_window_position(x: i32, y: i32) -> std::io::Result<()> {
    let pos = WindowPosition { x: Some(x), y: Some(y) };
    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    let json = serde_json::to_string_pretty(&pos)?;
    fs::write("data/window_position.json", json)
}

pub fn load_window_position() -> WindowPosition {
    let path = PathBuf::from("data/window_position.json");
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(pos) = serde_json::from_str::<WindowPosition>(&data) {
                return pos;
            }
        }
    }
    WindowPosition::default()
}
