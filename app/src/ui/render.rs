// ----- C:\Users\Joshua\Documents\Github\media-player\app\src\ui\render.rs -----

use iced::widget::{Column, Container, Row, Space};
use iced::{Element, Length, Alignment};

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

    // Create the bottom row with playlist (25%) and library (75%) side by side
    // Remove internal padding from playlist and library containers
    let playlist_section = playlist_view::view(playlists);
    let library_section = library_view::view(library);

    let bottom_row = Row::new()
        .push(
            Container::new(playlist_section.map(|_| Action::Playlist(core::PlaylistAction::Select(0))))
                .width(Length::FillPortion(1)) // 1 part (25%)
                .padding(0) // Remove padding
        )
        .push(
            Container::new(library_section.map(|_| Action::Library(core::LibraryAction::StartScan)))
                .width(Length::FillPortion(3)) // 3 parts (75%)
                .padding(0) // Remove padding
        )
        .spacing(0) // Remove spacing between playlist and library
        .padding(0) // Remove padding from the row itself
        .width(Length::Fill);

    // Main layout with player on top and the side-by-side views below
    let content = Column::new()
        // Map player actions to core actions
        .push(
            player_section.map(|ui_action| match ui_action {
                player_view::PlayerAction::Play => Action::Player(core::PlayerAction::Play("".to_string())),
                player_view::PlayerAction::Pause => Action::Player(core::PlayerAction::Pause),
                player_view::PlayerAction::Stop => Action::Player(core::PlayerAction::Stop),
                player_view::PlayerAction::None => Action::Player(core::PlayerAction::Stop),
            })
        )
        // Remove the Space element that was creating vertical gap
        .push(bottom_row)
        .spacing(0) // Remove spacing between player and bottom row
        .width(Length::Fill)
        .align_x(Alignment::Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(0) // Ensure the container doesn't add padding
        .into()
}