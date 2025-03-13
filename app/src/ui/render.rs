// ----- C:\Users\Joshua\Documents\Github\media-player\app\src\ui\render.rs -----

use iced::widget::{Column, Container};
use iced::{Element, Length};

use core::Action;
use core::player::PlayerState;
use core::playlist::PlaylistState;
use core::library::LibraryState;

use crate::ui::{player_view, playlist_view, library_view};

pub fn render<'a>(
    player_state: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState,
) -> Element<'a, Action> {
    let player_section = player_view::view(player_state);
    let playlist_section = playlist_view::view(playlists);
    let library_section = library_view::view(library);

    let content = Column::new()
        // Use closure to convert between PlayerAction types
        .push(player_section.map(|ui_action| match ui_action {
            player_view::PlayerAction::Play => Action::Player(core::PlayerAction::Play("".to_string())),
            player_view::PlayerAction::Pause => Action::Player(core::PlayerAction::Pause),
            player_view::PlayerAction::Stop => Action::Player(core::PlayerAction::Stop),
            player_view::PlayerAction::None => Action::Player(core::PlayerAction::Stop), // Default case
        }))
        .push(playlist_section.map(|_| Action::Playlist(core::PlaylistAction::Select(0))))
        .push(library_section.map(|_| Action::Library(core::LibraryAction::StartScan)))
        .spacing(20);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}