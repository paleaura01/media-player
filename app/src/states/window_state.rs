// app/src/states/window_state.rs
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WindowPosition {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

// Uncommented window position saving functionality
pub fn save_window_position(x: i32, y: i32) -> std::io::Result<()> {
    let pos = WindowPosition { x: Some(x), y: Some(y) };
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
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(pos) = serde_json::from_str::<WindowPosition>(&data) {
                return pos;
            }
        }
    }
    WindowPosition::default()
}

pub fn load_application_icon() -> Option<iced::window::Icon> {
    // Check if the icon file exists
    let asset_paths = [
        "app/assets/icon.ico",
        "assets/icon.ico",
        "icon.ico"
    ];
    
    let icon_path = asset_paths.iter()
        .find(|&path| std::path::Path::new(path).exists());
    
    if let Some(path) = icon_path {
        match std::fs::read(path) {
            Ok(icon_bytes) => {
                match iced::window::icon::from_file_data(&icon_bytes, None) {
                    Ok(icon) => Some(icon),
                    Err(err) => {
                        log::error!("Failed to create icon from data: {}", err);
                        None
                    }
                }
            },
            Err(err) => {
                log::error!("Failed to read icon file {}: {}", path, err);
                None
            }
        }
    } else {
        log::warn!("Application icon not found in any of the expected locations");
        None
    }
}

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
        icon: load_application_icon(),
        ..WindowSettings::default()
    }
}