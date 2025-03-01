// src/main.rs
use log::info;
use iced::Application;
use media_player::gui::MediaPlayer;

fn main() {
    // Ensure logger is only initialized once at the start of the program.
    if let Err(_) = env_logger::try_init() {
        // If already initialized, ignore the error.
    }
    
    info!("Media Player starting up...");

    // Launch the GUI application
    MediaPlayer::run(iced::Settings::default())
        .expect("Failed to launch GUI");
}