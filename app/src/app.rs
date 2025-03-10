// app/src/app.rs
use iced::{Element, Task};
use core::Action;
use crate::app_state::MediaPlayer;
use crate::ui;

#[derive(Debug, Clone)]
pub enum Message {
    Action(Action),
}

pub fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    match message {
        Message::Action(action) => {
            state.handle_action(action);
        }
    }
    // No asynchronous work, so return an empty task.
    Task::none()
}

pub fn view(state: &MediaPlayer) -> Element<Message> {
    // Call the UI render function directly.
    let ui_element = ui::render(&state.player_state, &state.playlists, &state.library);
    // Map the UI element to our Message type.
    ui_element.0.map(Message::Action)
}
