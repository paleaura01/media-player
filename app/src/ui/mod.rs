// app/src/ui/mod.rs
pub mod theme;
pub mod render;
pub mod player_view;
pub mod playlist_view;
pub mod library_view;

// Helper function: Combines your sub-views into a full UI layout.
use core::player::PlayerState;
use core::playlist::PlaylistState;
use core::library::LibraryState;
use core::Action;
use iced::{Element, widget::{Column, Container}, Length};

pub fn show_full_ui<'a>(
    player: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState,
) -> Element<'a, Action> {
    let player_element = player_view::view(player);
    let playlist_element = playlist_view::view(playlists);
    let library_element = library_view::view(library);

    let content = Column::new()
        // Use closure to convert between PlayerAction types
        .push(player_element.map(|ui_action| match ui_action {
            player_view::PlayerAction::Play => Action::Player(core::PlayerAction::Play("".to_string())),
            player_view::PlayerAction::Pause => Action::Player(core::PlayerAction::Pause),
            player_view::PlayerAction::Stop => Action::Player(core::PlayerAction::Stop),
            player_view::PlayerAction::None => Action::Player(core::PlayerAction::Stop), // Default case
        }))
        .push(playlist_element.map(|_| Action::Playlist(core::PlaylistAction::Select(0))))
        .push(library_element.map(|_| Action::Library(core::LibraryAction::StartScan)))
        .spacing(20);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}