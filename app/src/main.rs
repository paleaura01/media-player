#![windows_subsystem = "windows"] // Hides the console on Windows

mod ui;
mod states; // Changed from mod app_state
mod application;

// Initialize logging
fn setup_logging() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
}

fn main() -> iced::Result {
    // Set up logging
    setup_logging();
    
    // Run the application using the implementation in application.rs
    application::run()
}