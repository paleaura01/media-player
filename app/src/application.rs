use iced::{Element, Subscription}; // Removed the import of `run`
use crate::ui;
use crate::states::playlist_state::PlaylistViewState; 
use crate::states::app_state::MediaPlayer; // Main application state

// Import the PlaylistAction type from the UI module.
use crate::ui::playlist_view::PlaylistAction;

#[derive(Debug, Clone)]
pub enum Message {
    /// Core action messages.
    Action(core::Action),
    /// Playlist messages from the UI (of type PlaylistAction).
    Playlist(PlaylistAction),
}

// Global static for the playlist UI state.
static mut PLAYLIST_STATE: Option<PlaylistViewState> = None;

fn get_playlist_state() -> &'static mut PlaylistViewState {
    unsafe {
        if PLAYLIST_STATE.is_none() {
            PLAYLIST_STATE = Some(PlaylistViewState::new());
        }
        PLAYLIST_STATE.as_mut().unwrap()
    }
}

// Updated update function: now returns nothing.
fn update(state: &mut MediaPlayer, message: Message) {
    match message {
        Message::Action(action) => {
            state.handle_action(action);
        }
        Message::Playlist(action) => {
            // Use the playlist UI state to handle the raw PlaylistAction,
            // converting it to a core Action.
            let core_action = get_playlist_state().handle_action(action);
            state.handle_action(core_action);
        }
    }
}

fn view(state: &MediaPlayer) -> Element<Message> {
    let playlist_state = get_playlist_state();

    // Render the UI with playlist editing support.
    // render_with_state now returns an Element<PlaylistAction>.
    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        playlist_state,
    );

    // Map the UI's PlaylistAction into our Message::Playlist variant.
    rendered.map(Message::Playlist)
}

fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    Subscription::none()
}

pub fn run() -> iced::Result {
    // Call iced::run (fully-qualified) to start the application.
    iced::run("Media Player", update, view)
}
