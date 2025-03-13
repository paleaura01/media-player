use iced::{Element, Subscription, Task};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::key::Named;

// Import the PlaylistAction type from the UI module
use crate::ui::playlist_view::PlaylistAction;

#[derive(Debug, Clone)]
pub enum Message {
    /// Core action messages
    Action(core::Action),
    /// Playlist messages from the UI (of type PlaylistAction)
    Playlist(PlaylistAction),
    /// Window events
    WindowClosed { x: i32, y: i32 },
}

// Updated to return Task<Message> instead of void
fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    match message {
        Message::Action(action) => {
            state.handle_action(action);
            Task::none()
        }
        Message::Playlist(action) => {
            let core_action = state.playlist_view_state.handle_action(action);
            state.handle_action(core_action);
            Task::none()
        }
        Message::WindowClosed { x, y } => {
            if let Err(e) = window_state::save_window_position(x, y) {
                log::error!("Failed to save window position: {}", e);
            }
            Task::none()
        }
    }
}

fn view(state: &MediaPlayer) -> Element<Message> {
    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        &state.playlist_view_state,
    );

    rendered.map(Message::Playlist)
}

// Updated subscription to handle keyboard events along with window events
fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    iced::event::listen().map(|event| {
        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                match key {
                    iced::keyboard::Key::Named(Named::Space) => {
                        Message::Action(core::Action::Player(core::PlayerAction::Pause))
                    },
                    iced::keyboard::Key::Named(Named::Escape) => {
                        Message::Action(core::Action::Player(core::PlayerAction::Stop))
                    },
                    _ => Message::Playlist(PlaylistAction::None)
                }
            },
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                Message::WindowClosed { x: 100, y: 100 }
            },
            iced::Event::Window(iced::window::Event::Moved(position)) => {
                let x = position.x as i32;
                let y = position.y as i32;
                Message::WindowClosed { x, y }
            },
            _ => {
                Message::Playlist(PlaylistAction::None)
            }
        }
    })
}

pub fn run() -> iced::Result {
    iced::application("Media Player", update, view)
        .subscription(subscription)
        .window(window_state::window_settings())
        .theme(|_state| ui::theme::dark_theme())
        .run()
}