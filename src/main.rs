// src/main.rs
use std::env;
use log::info;
use media_player::Player;

fn main() {
    // Ensure logger is only initialized once at the start of the program.
    if let Err(_) = env_logger::try_init() {
        // If already initialized, ignore the error.
    }
    
    info!("Media Player starting up...");

    // Accept an audio file path from command-line arguments or use a default.
    let args: Vec<String> = env::args().collect();
    let audio_file = args.get(1).map(|s| s.as_str()).unwrap_or("sample.mp3");
    info!("Attempting to play file: {}", audio_file);

    // Create a new Player instance and attempt to play the specified audio file.
    let mut player = Player::new();
    match player.play(audio_file) {
        Ok(_) => {
            info!("Playback started. Playing '{}'...", audio_file);
            player.wait_until_finished();
            info!("Playback finished.");
        }
        Err(e) => {
            log::error!("Failed to start playback: {}", e);
        }
    }
}
