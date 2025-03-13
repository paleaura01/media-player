// ----- C:\Users\Joshua\Documents\Github\media-player\app\src\window_manager.rs -----

use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WindowPosition {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

// If you want to store window position on close, uncomment & integrate this
/* 
pub fn save_window_position(x: i32, y: i32) -> std::io::Result<()> {
    let pos = WindowPosition { x: Some(x), y: Some(y) };
    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    let json = serde_json::to_string_pretty(&pos)?;
    fs::write("data/window_position.json", json)
}
*/

// We do use load_window_position in window_settings, so we keep it
fn load_window_position() -> WindowPosition {
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

// Provide a public function for the Iced window settings
pub fn window_settings() -> iced::window::Settings {
    use iced::window::{Settings as WindowSettings, Position};
    use iced::{Size, Point};

    let pos = load_window_position();
    let position = if let (Some(x), Some(y)) = (pos.x, pos.y) {
        Position::Specific(Point::new(x as f32, y as f32))
    } else {
        Position::Centered
    };

    WindowSettings {
        size: Size {
            width: 1000.0,
            height: 700.0,
        },
        position,
        ..WindowSettings::default()
    }
}
