// ----- C:\Users\Joshua\Documents\Github\media-player\app\src\application.rs -----

use iced::{
    application,
    Element,
    Subscription,
};

use core::Action;
use crate::{ui, window_manager};

// Import MediaPlayer from your player_ui module (adjust path if needed)
use player_ui::app_state::MediaPlayer;

#[derive(Debug, Clone)]
pub enum Message {
    Action(Action),
}

fn update(state: &mut MediaPlayer, message: Message) {
    match message {
        Message::Action(action) => {
            state.handle_action(action);
        }
    }
}

fn view(state: &MediaPlayer) -> Element<Message> {
    let rendered = ui::render::render(
        &state.player_state,
        &state.playlists,
        &state.library,
    );
    rendered.map(Message::Action)
}

fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    Subscription::none()
}

pub fn run() -> iced::Result {
    application("Media Player", update, view)
        .subscription(subscription)
        .window(window_manager::window_settings())
        .theme(|_state| ui::theme::dark_theme())
        .run()
}
