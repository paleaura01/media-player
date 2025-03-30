// app/src/ui/render.rs
use iced::widget::{Column, Container, Row, container, text, scrollable, Space, horizontal_rule, button};
use iced::{Element, Length, Background, Alignment, Theme, widget::svg, Color};
use crate::ui::theme::{
    library_container_style,
    playlist_container_style,
    now_playing_container_style,
    player_container_style,
    DARK_BG_COLOR,
    GREEN_COLOR,
};

// Fixed import to use the correct path for PlayerState
use core::player::state::PlayerState;
use core::playlist::PlaylistState;
use core::library::LibraryState;

use crate::ui::{player_view, playlist_view, library_view};
use crate::states::playlist_state::PlaylistViewState; 
use crate::ui::playlist_view::PlaylistAction;

fn load_icon(name: &str) -> svg::Svg<iced::Theme> {
    let base_path = std::env::current_dir().unwrap_or_default();
    let icon_path = base_path.join("app").join("assets").join("icons").join(name);
    println!("Loading icon from: {}", icon_path.display());

    svg::Svg::new(svg::Handle::from_path(icon_path))
}

/// Enhanced render function with updated layout & action mapping
pub fn render_with_state<'a>(
    player_state: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState,
    playlist_view_state: &'a PlaylistViewState,
    status_message: &'a Option<String>, // Status message for user feedback
) -> Element<'a, playlist_view::PlaylistAction> {
    // Player section
    let player_section = player_view::view(player_state);
    
    // Create the three panels for the main content area
    let playlist_section = playlist_view::view_with_state(playlists, playlist_view_state);
    let library_section = library_view::view_with_search(library);
    let now_playing_section = create_now_playing_section(playlists, player_state);
    
    // Map player actions -> playlist actions
    let player_container = Container::new(
        player_section.map(|ui_action| {
            // Debug logging to trace action mapping
            match &ui_action {
                player_view::PlayerAction::Seek(pos) => {
                    println!("██ DEBUG: Mapping UI Seek({:.4}) to PlaylistAction::Seek in render.rs", pos);
                },
                _ => {}
            }
            
            match ui_action {
                player_view::PlayerAction::Play => 
                    PlaylistAction::PlayerControl(core::PlayerAction::Resume),
                player_view::PlayerAction::Pause => 
                    PlaylistAction::PlayerControl(core::PlayerAction::Pause),
                player_view::PlayerAction::SkipForward => 
                    PlaylistAction::PlayerControl(core::PlayerAction::SkipForward(10.0)),
                player_view::PlayerAction::SkipBackward => 
                    PlaylistAction::PlayerControl(core::PlayerAction::SkipBackward(10.0)),
                player_view::PlayerAction::Next => 
                    PlaylistAction::PlayerControl(core::PlayerAction::NextTrack),
                player_view::PlayerAction::Previous => 
                    PlaylistAction::PlayerControl(core::PlayerAction::PreviousTrack),
                player_view::PlayerAction::VolumeChange(v) => 
                    PlaylistAction::PlayerControl(core::PlayerAction::SetVolume(v)),
                player_view::PlayerAction::Seek(pos) => {
                    println!("██ DEBUG: Creating PlaylistAction::Seek({:.4}) in render.rs", pos);
                    PlaylistAction::Seek(pos) // <- CRITICAL: direct Seek action mapping
                },
                player_view::PlayerAction::Shuffle =>
                    PlaylistAction::PlayerControl(core::PlayerAction::Shuffle),
                player_view::PlayerAction::UpdateProgress(pos) => {
                    // For continuous updates during slider dragging
                    PlaylistAction::UpdateProgress(pos)
                },
            }
        })
    )
    .width(Length::Fill)
    .style(player_container_style());

    // Left panel - Playlists (15%)
    let playlist_container = Container::new(
        playlist_section // Use directly without mapping
    )
    .width(Length::FillPortion(15))
    .height(Length::Fill)
    .style(playlist_container_style());

    // Middle panel - Now Playing (25%)
    let now_playing_container = Container::new(
        now_playing_section // Use directly without mapping
    )
    .width(Length::FillPortion(25))
    .height(Length::Fill)
    .style(now_playing_container_style());

    // Right panel - Library - Map to Library PlaylistAction
    let library_container = Container::new(
        library_section.map(PlaylistAction::Library) // Map to PlaylistAction::Library
    )
    .width(Length::FillPortion(60))
    .height(Length::Fill)
    .style(library_container_style());

    // Main content area
    let content_row = Row::new()
        .push(playlist_container)
        .push(now_playing_container)
        .push(library_container)
        .spacing(1)
        .height(Length::Fill)
        .width(Length::Fill);

    // Overall layout with top player container and optional status message
    let mut content = Column::new()
        .push(player_container)
        .push(content_row)
        .spacing(1)
        .width(Length::Fill)
        .height(Length::Fill);
        
    // Add status message if present
    if let Some(message) = status_message {
        content = content.push(
            Container::new(
                text(message).size(14)
                    .style(|_: &Theme| text::Style {
                        color: Some(Color::from_rgb(1.0, 1.0, 0.8)), // Slightly different color for visibility
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .padding(8)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                border: iced::Border {
                    color: GREEN_COLOR,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(GREEN_COLOR),
                ..Default::default()
            })
        );
    }

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

// Helper: now playing section
fn create_now_playing_section<'a>(
    playlists: &'a PlaylistState, 
    player_state: &'a PlayerState
) -> Element<'a, PlaylistAction> {
    let title = text("Now Playing")
        .size(20)
        .style(|_| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        });

    // Build a complete content column step by step
    let mut content = Column::new().push(title);

    if let Some(idx) = playlists.selected {
        if idx < playlists.playlists.len() {
            let playlist = &playlists.playlists[idx];
            
            // Add playlist name to content
            content = content.push(text(&playlist.name).size(16))
                .push(Space::with_height(10));
            
            // Create tracks view
            let tracks_column = Column::new()
                .spacing(5)
                .padding(10)
                .push(text(format!("{} tracks", playlist.tracks.len())).size(14))
                .push(horizontal_rule(1));
                
            // Build track list
            let track_list = playlist.tracks.iter().enumerate().fold(
                Column::new().spacing(4),
                |column, (track_idx, track)| {
                    let track_title = track.title.clone().unwrap_or_else(|| track.path.clone());
                    
                    // Check if this track is the currently playing track
                    let is_current_track = if let Some(current_path) = &player_state.current_track {
                        current_path == &track.path
                    } else {
                        false
                    };
                    
                    // Create track row with play button and delete button
                    let track_row = Row::new()
                        .push(
                            button(
                                Row::new()
                                    .push(text(format!("{}. ", track_idx + 1)).size(14))
                                    .push(
                                        text(format!("{} (played {})", 
                                            track_title, 
                                            track.play_count))
                                        .size(14)
                                    )
                                    .spacing(5)
                            )
                            .padding(5)
                            .width(Length::Fill)
                            .style(|_theme, _| button::Style {
                                background: None,
                                text_color: GREEN_COLOR,
                                ..Default::default()
                            })
                            .on_press(PlaylistAction::PlayTrack(playlist.id, track_idx))
                        )
                        .push(
                            button(
                                load_icon("ph--x-square-bold.svg")
                                    .width(16)
                                    .height(16)
                            )
                            .padding(5)
                            .on_press(PlaylistAction::RemoveTrack(playlist.id, track_idx))
                            .style(|_theme, _| button::Style {
                                background: None,
                                ..Default::default()
                            })
                        )
                        .spacing(5)
                        .align_y(Alignment::Center);
                    
                    // Add a network indicator for network paths
                    let track_row = if track.path.starts_with("\\\\") || track.path.contains("://") {
                        Row::new()
                            .push(
                                text("🌐 ").size(14)
                                    .style(|_: &Theme| text::Style {
                                        color: Some(Color::from_rgb(0.6, 0.8, 1.0)),
                                        ..Default::default()
                                    })
                            )
                            .push(track_row)
                            .spacing(2)
                            .align_y(Alignment::Center)
                    } else {
                        Row::new()
                            .push(Space::with_width(Length::Fixed(18.0)))
                            .push(track_row)
                            .spacing(2)
                            .align_y(Alignment::Center)
                    };
                    
                    // Wrap in a container that can be highlighted
                    let track_container = container(track_row)
                        .width(Length::Fill)
                        .padding(2)
                        .style(move |_: &Theme| container::Style {
                            background: if is_current_track {
                                Some(Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.15)))
                            } else {
                                None
                            },
                            ..Default::default()
                        });
                    
                    column.push(track_container)
                }
            );
            
            // Add track list to main column
            let tracks_column = tracks_column.push(track_list);
            
            // Add scrollable container with tracks
            content = content.push(scrollable(tracks_column));
        } else {
            content = content.push(text("No playlist selected").size(16));
        }
    } else {
        content = content.push(text("No playlist selected").size(16));
    }

    content
        .spacing(5)
        .padding(10)
        .into()
}