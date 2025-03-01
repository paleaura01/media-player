extern crate serde;

use iced::{Application, Settings, window};
use media_gui::MediaPlayer;
use confy;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WindowConfig {
    x: i32,
    y: i32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self { x: 100, y: 100 }
    }
}

fn main() {
    let _ = env_logger::try_init();

    // Load the saved window position (or use default values)
    let config: WindowConfig = confy::load("media-player", None).unwrap_or_default();

    let settings = Settings {
        window: window::Settings {
            position: window::Position::Specific(config.x, config.y),
            ..Default::default()
        },
        ..Default::default()
    };

    MediaPlayer::run(settings)
        .expect("Failed to launch GUI");
}
