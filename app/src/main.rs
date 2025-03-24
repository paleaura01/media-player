// app/src/main.rs - Update to use proper file logging
#![windows_subsystem = "windows"] // Hides the console on Windows

mod ui;
mod states;
mod application;

use std::fs::File;
use log::{LevelFilter, info};
use std::io::Write;

// Enhanced logging setup with file output
fn setup_logging() {
    // Create logs directory if it doesn't exist
    if !std::path::Path::new("logs").exists() {
        std::fs::create_dir("logs").expect("Failed to create logs directory");
    }
    
    // Set up file logger with timestamp in the filename
    let now = chrono::Local::now();
    let log_filename = format!("logs/player_log_{}.txt", now.format("%Y%m%d_%H%M%S"));
    
    // Create log file with buffered writing
    let log_file = File::create(&log_filename).expect("Failed to create log file");
    
    // Configure the logger to write both to file and console
    let mut builder = env_logger::Builder::new();
    builder
        .format(|buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                buf,
                "[{}] {} [{}:{}] {}",
                timestamp,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter(None, LevelFilter::Info)
        .init();
    
    info!("Logging initialized to file: {}", log_filename);
    
    // Also create a symlink/copy to "latest.log" for easier access
    let latest_log_path = "logs/latest.log";
    if std::path::Path::new(latest_log_path).exists() {
        let _ = std::fs::remove_file(latest_log_path);
    }
    
    // Try to create a symlink, but fall back to copying the file if symlinks fail
    let copy_result = std::fs::copy(&log_filename, latest_log_path);
    if let Err(e) = copy_result {
        eprintln!("Warning: Could not create latest.log symlink: {}", e);
    }
}

fn main() -> iced::Result {
    // Set up logging
    setup_logging();
    
    // Log application start
    info!("Application starting...");
    
    // Run the application using the implementation in application.rs
    application::run()
}