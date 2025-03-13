// ----- C:\Users\Joshua\Documents\Github\media-player\app\src\ui\render.rs -----

use iced::widget::{Column, Container, Row, container};
use iced::{Element, Length, Background, Border};

use core::Action;
use core::player::PlayerState;
use core::playlist::PlaylistState;
use core::library::LibraryState;

use crate::ui::{player_view, playlist_view, library_view};
use crate::ui::theme::{
    borderless_dark_container_style, 
    library_container_style,
    GREEN_COLOR, 
    BLACK_COLOR
};

pub fn render<'a>(
    player_state: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState,
) -> Element<'a, Action> {
    let player_section = player_view::view(player_state);
    let playlist_section = playlist_view::view(playlists);
    let library_section = library_view::view(library);

    // Create player section with no border
    let player_container = Container::new(
        player_section.map(|ui_action| match ui_action {
            player_view::PlayerAction::Play => Action::Player(core::PlayerAction::Play("".to_string())),
            player_view::PlayerAction::Pause => Action::Player(core::PlayerAction::Pause),
            player_view::PlayerAction::Stop => Action::Player(core::PlayerAction::Stop),
            player_view::PlayerAction::None => Action::Player(core::PlayerAction::Stop),
        })
    )
    .width(Length::Fill)
    .style(borderless_dark_container_style());

    // Playlist section with no border
    let playlist_container = Container::new(
        playlist_section.map(|_| Action::Playlist(core::PlaylistAction::Select(0)))
    )
    .width(Length::FillPortion(1))
    .style(borderless_dark_container_style());

    // Library section with border to divide it from playlist
    let library_container = Container::new(
        library_section.map(|_| Action::Library(core::LibraryAction::StartScan))
    )
    .width(Length::FillPortion(3))
    .style(library_container_style());

    // The bottom row containing playlist and library
    let bottom_row = Row::new()
        .push(playlist_container)
        .push(library_container)
        .spacing(0)
        .width(Length::Fill);

    // Main layout with player on top and bottom row below
    let content = Column::new()
        .push(player_container)
        .push(bottom_row)
        .spacing(0)
        .width(Length::Fill);

    // First, create an inner container with the content
    let inner_container = Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0);

    // Then wrap it in an outer container with padding and a border
    // This creates a visual border effect that will be clearly visible
    Container::new(inner_container)
        .padding(3) // This creates space for the border
        .style(|_| container::Style {
            background: Some(Background::Color(GREEN_COLOR)), // The padding area becomes the border
            text_color: Some(GREEN_COLOR),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}