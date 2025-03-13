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

    // Create the bottom row with playlist and library side by side
    let playlist_section = playlist_view::view(playlists);
    let library_section = library_view::view(library);

    let bottom_row = Row::new()
        .push(
            Container::new(playlist_section.map(|_| Action::Playlist(core::PlaylistAction::Select(0))))
                .width(Length::FillPortion(1))
                .padding(15)
        )
        .push(
            Container::new(library_section.map(|_| Action::Library(core::LibraryAction::StartScan)))
                .width(Length::FillPortion(1))
                .padding(15)
        )
        .spacing(20)
        .padding(10)
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
        .push(Space::with_height(20)) // Just pass the integer directly
        .push(bottom_row)
        .spacing(10)
        .width(Length::Fill)
        .align_x(Alignment::Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}