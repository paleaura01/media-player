#![windows_subsystem = "windows"] // Hides the console on Windows

mod ui;
mod app_state; 
mod application;
mod window_manager;

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