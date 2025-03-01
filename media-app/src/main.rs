use iced::Application;
use iced::window;
use iced::Settings;
use media_gui::MediaPlayer;
use media_core::settings::AppSettings;

fn main() {
    let _ = env_logger::try_init();
    
    // Load application settings
    let settings = AppSettings::load();
    
    // Configure window settings
    let window_settings = window::Settings {
        // Use saved position or default to centered
        position: settings.window_position
            .map(|(x, y)| window::Position::Specific(x, y))
            .unwrap_or(window::Position::Centered),
        // Other window settings as default
        ..Default::default()
    };
    
    // Create application settings
    let app_settings = Settings {
        window: window_settings,
        ..Settings::default()
    };
    
    // Run the application
    MediaPlayer::run(app_settings)
        .expect("Failed to launch GUI");
}