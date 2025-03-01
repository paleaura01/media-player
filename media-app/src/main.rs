use iced::Application;
use media_gui::MediaPlayer;

fn main() {
    let _ = env_logger::try_init();
    MediaPlayer::run(iced::Settings::default())
        .expect("Failed to launch GUI");
}
