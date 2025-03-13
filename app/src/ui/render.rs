use iced::widget::{Column, Container, Row, container};
use iced::{Element, Length, Background};

use core::player::PlayerState;
use core::playlist::PlaylistState;
use core::library::LibraryState;

use crate::ui::{player_view, playlist_view, library_view};
use crate::states::playlist_state::PlaylistViewState; 
use crate::ui::theme::{
    borderless_dark_container_style, 
    library_container_style,
    DARK_GREEN_COLOR,
};

/// Enhanced render with playlist editing state.
/// Returns an Element with messages of type `playlist_view::PlaylistAction`.
pub fn render_with_state<'a>(
    player_state: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState,
    playlist_view_state: &'a PlaylistViewState,
) -> Element<'a, playlist_view::PlaylistAction> {
    use crate::ui::playlist_view;

    let player_section = player_view::view(player_state);
    let playlist_section = playlist_view::view_with_state(playlists, playlist_view_state);
    let library_section = library_view::view(library);

    let player_container = Container::new(
        player_section.map(|ui_action| match ui_action {
            player_view::PlayerAction::Play => playlist_view::PlaylistAction::None,
            player_view::PlayerAction::Pause => playlist_view::PlaylistAction::None,
            player_view::PlayerAction::Stop => playlist_view::PlaylistAction::None,
        })
    )
    .width(Length::Fill)
    .style(borderless_dark_container_style());

    // Forward the playlist view actions.
    let playlist_container = Container::new(
        playlist_section.map(|action| action)
    )
    .width(Length::FillPortion(1))
    .style(borderless_dark_container_style());

    let library_container = Container::new(
        library_section.map(|_| playlist_view::PlaylistAction::None)
    )
    .width(Length::FillPortion(3))
    .style(library_container_style());

    let bottom_row = Row::new()
        .push(playlist_container)
        .push(library_container)
        .spacing(0)
        .width(Length::Fill);

    let content = Column::new()
        .push(player_container)
        .push(bottom_row)
        .spacing(0)
        .width(Length::Fill);

    let inner_container = Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0);

    Container::new(inner_container)
        .padding(1)
        .style(|_| container::Style {
            background: Some(Background::Color(DARK_GREEN_COLOR)),
            text_color: Some(DARK_GREEN_COLOR),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
