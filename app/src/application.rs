// app/src/application.rs
use iced::{Element, Subscription, Task, Point};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::key::Named;
use std::path::PathBuf;

// Import message types from UI modules
use crate::ui::playlist_view::PlaylistAction;
use crate::ui::library_view::LibraryMessage;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    /// Core action messages
    Action(core::Action),
    /// Playlist messages from the UI
    Playlist(PlaylistAction),
    /// Library messages
    Library(LibraryMessage),
    /// Folder selection result
    FolderSelected(Option<PathBuf>),
    /// Window events
    WindowClosed { x: i32, y: i32 },
    /// Mouse position for hover detection
    MousePosition(Point),
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
        Message::Library(LibraryMessage::AddMusicFolder) => {
            // Ideally you'd use rfd here, but for now we'll manually implement
            // by using a core library action
            state.handle_action(core::Action::Library(
                core::LibraryAction::StartScan
            ));
            Task::none()
        }
        Message::Library(LibraryMessage::None) => {
            Task::none()
        }
        Message::Library(LibraryMessage::ToggleView) => {
            // Handle the toggle view action here
            // For now, we'll just do nothing as the view toggle functionality
            // isn't fully implemented
            Task::none()
        }
        Message::FolderSelected(Some(path)) => {
            if let Some(path_str) = path.to_str() {
                state.handle_action(core::Action::Library(
                    core::LibraryAction::AddScanDirectory(path_str.to_string())
                ));
                state.handle_action(core::Action::Library(
                    core::LibraryAction::StartScan
                ));
            }
            Task::none()
        }
        Message::FolderSelected(None) => {
            Task::none()
        }
        Message::WindowClosed { x, y } => {
            if let Err(e) = window_state::save_window_position(x, y) {
                log::error!("Failed to save window position: {}", e);
            }
            Task::none()
        }
        Message::MousePosition(_position) => {
            // We don't need to track individual mouse positions anymore
            // as we're using selection state instead of hover
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
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Message::MousePosition(position)
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