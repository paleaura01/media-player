// src/main.rs
use iced::Application;
use media_player::gui::MediaPlayer;

fn main() {
    // Initialize logging
    let _ = env_logger::try_init();
    
    // Start the GUI
    MediaPlayer::run(iced::Settings::default())
        .expect("Failed to launch GUI");
}