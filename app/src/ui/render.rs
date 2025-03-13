use iced::widget::{Column, Container, Row, container, text, scrollable, Space, horizontal_rule};
use iced::{Element, Length, Background, Border};
use crate::ui::theme::{
    library_container_style,
    playlist_container_style,
    now_playing_container_style,
    DARK_BG_COLOR,
    DARK_GREEN_COLOR,
    GREEN_COLOR,
};

use core::player::PlayerState;
use core::playlist::PlaylistState;
use core::library::LibraryState;

use crate::ui::{player_view, playlist_view, library_view};
use crate::states::playlist_state::PlaylistViewState; 

/// Enhanced render with updated layout to match reference design
pub fn render_with_state<'a>(
    player_state: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState,
    playlist_view_state: &'a PlaylistViewState,
) -> Element<'a, playlist_view::PlaylistAction> {
    use crate::ui::playlist_view::PlaylistAction;
    
    // Top player section
    let player_section = player_view::view(player_state);
    
    // Create the three panels for the main content area
    let playlist_section = playlist_view::view_with_state(playlists, playlist_view_state);
    let library_section = library_view::view_with_search(library);
    let now_playing_section = create_now_playing_section(playlists);
    
    // Player container at the top
    let player_container = Container::new(
        player_section.map(|ui_action| match ui_action {
            player_view::PlayerAction::Play => PlaylistAction::None,
            player_view::PlayerAction::Pause => PlaylistAction::None,
            player_view::PlayerAction::Stop => PlaylistAction::None,
            player_view::PlayerAction::SkipForward => PlaylistAction::None,
            player_view::PlayerAction::SkipBackward => PlaylistAction::None,
            player_view::PlayerAction::Next => PlaylistAction::None,
            player_view::PlayerAction::Previous => PlaylistAction::None,
            player_view::PlayerAction::VolumeChange(_) => PlaylistAction::None,
            player_view::PlayerAction::Seek(_) => PlaylistAction::None,
        })
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(DARK_BG_COLOR)),
        border: Border {
            color: DARK_GREEN_COLOR,
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: Some(GREEN_COLOR),
        ..Default::default()
    });

    // Left panel - Playlists (15%)
    let playlist_container = Container::new(
        playlist_section.map(|action| action)
    )
    .width(Length::FillPortion(15))
    .height(Length::Fill)
    .style(playlist_container_style());

    // Middle panel - Now Playing (25%)
    let now_playing_container = Container::new(
        now_playing_section.map(|_| PlaylistAction::None)
    )
    .width(Length::FillPortion(25))
    .height(Length::Fill)
    .style(now_playing_container_style());

    // Right panel - Library (60%)
    let library_container = Container::new(
        library_section.map(|_| PlaylistAction::None)
    )
    .width(Length::FillPortion(60))
    .height(Length::Fill)
    .style(library_container_style());

    // Main content area with three panels
    let content_row = Row::new()
        .push(playlist_container)
        .push(now_playing_container)
        .push(library_container)
        .spacing(1)
        .height(Length::Fill)
        .width(Length::Fill);

    // Main layout with player at top and content below
    let content = Column::new()
        .push(player_container)
        .push(content_row)
        .spacing(1)
        .width(Length::Fill)
        .height(Length::Fill);

    Container::new(content)
        .padding(0)
        .style(|_| container::Style {
            background: Some(Background::Color(DARK_BG_COLOR)),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// Helper function to create the now playing section
fn create_now_playing_section<'a>(playlists: &'a PlaylistState) -> Element<'a, ()> {
    let title = text("Now Playing")
        .size(20)
        .style(|_| text::Style {
            color: Some(DARK_GREEN_COLOR),
            ..Default::default()
        });

    let content = if let Some(idx) = playlists.selected {
        if idx < playlists.playlists.len() {
            let playlist = &playlists.playlists[idx];
            
            let tracks = scrollable(
                Column::new()
                    .spacing(5)
                    .padding(10)
                    .push(text(format!("{} tracks", playlist.tracks.len())).size(14))
                    .push(horizontal_rule(1))
                    .push(
                        playlist.tracks.iter().enumerate().fold(
                            Column::new().spacing(4),
                            |column, (idx, track)| {
                                let track_title = track.title.clone().unwrap_or_else(|| track.path.clone());
                                column.push(
                                    Row::new()
                                        .push(text(format!("{}. ", idx + 1)).size(14))
                                        .push(text(track_title).size(14))
                                        .spacing(5)
                                )
                            }
                        )
                    )
            );
            
            Column::new()
                .push(title)
                .push(text(&playlist.name).size(16))
                .push(Space::with_height(10))
                .push(tracks)
                .spacing(5)
                .padding(10)
                .into()
        } else {
            Column::new()
                .push(title)
                .push(text("No playlist selected").size(16))
                .spacing(10)
                .padding(10)
                .into()
        }
    } else {
        Column::new()
            .push(title)
            .push(text("No playlist selected").size(16))
            .spacing(10)
            .padding(10)
            .into()
    };
    
    content
}