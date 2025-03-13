use iced::{Element, Subscription, Task};
use crate::ui;
use crate::states::window_state;  // Removed playlist_state from imports
use crate::states::playlist_state::PlaylistViewState; 
use crate::states::app_state::MediaPlayer;

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

// Global static for the playlist UI state
static mut PLAYLIST_STATE: Option<PlaylistViewState> = None;

fn get_playlist_state() -> &'static mut PlaylistViewState {
    unsafe {
        if PLAYLIST_STATE.is_none() {
            PLAYLIST_STATE = Some(PlaylistViewState::new());
        }
        PLAYLIST_STATE.as_mut().unwrap()
    }
}

// Updated to return Task<Message> instead of void
fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    match message {
        Message::Action(action) => {
            state.handle_action(action);
            Task::none()
        }
        Message::Playlist(action) => {
            let core_action = get_playlist_state().handle_action(action);
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
    let playlist_state = get_playlist_state();

    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        playlist_state,
    );

    rendered.map(Message::Playlist)
}

// Added subscription to window events with corrected pattern matching
fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    iced::event::listen().map(|event| {
        match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                // Get current window position before closing
                // Note: This is a simplified approach - in production you'd get actual position
                Message::WindowClosed { x: 100, y: 100 }
            }
            iced::Event::Window(iced::window::Event::Moved(position)) => {
                // Corrected: Extract x and y from the Point struct
                let x = position.x as i32;
                let y = position.y as i32;
                Message::WindowClosed { x, y }
            }
            _ => {
                // Default message for other events
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