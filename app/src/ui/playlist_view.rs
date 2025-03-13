// app/src/ui/playlist_view.rs
use iced::widget::{button, column, container, row, text, scrollable, Space, text_input};
use iced::{Alignment, Element, Length, Theme};
use core::playlist::PlaylistState;
use crate::ui::theme::GREEN_COLOR;
use crate::states::playlist_state::PlaylistViewState;

#[derive(Debug, Clone)]
pub enum PlaylistAction {
    Create(String),
    Delete(u32),
    Select(u32),
    StartEditing(u32, String),
    EditingName(String),
    FinishEditing,
    None,
    HoverPlaylist(Option<u32>),
}

// Enhanced view with editing state and hover delete functionality
pub fn view_with_state<'a>(playlist_state: &'a PlaylistState, view_state: &'a PlaylistViewState) -> Element<'a, PlaylistAction> {
    let header = text("Playlists")
        .size(20)
        .style(|_: &Theme| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        });
    
    let add_button = button(
        text("+ Add Playlist").style(|_: &Theme| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        })
    )
    .padding(5)
    .on_press(PlaylistAction::Create("New Playlist".to_string()))
    .style(|_theme, _| button::Style {
        background: None,
        text_color: GREEN_COLOR,
        ..Default::default()
    });

    // Create playlist rows
    let playlist_rows = column(
        playlist_state.playlists.iter().enumerate().map(|(idx, playlist)| {
            let id = idx as u32;
            let is_hovered = view_state.hovered_playlist_id == Some(id);
            let is_selected = Some(idx) == playlist_state.selected;
            
            // Check if this playlist is being edited
            let row_content = if view_state.is_editing(id) {
                // Editing mode - show text input
                row![
                    text_input("Enter name", view_state.edit_value())
                        .on_input(PlaylistAction::EditingName)
                        .on_submit(PlaylistAction::FinishEditing)
                        .padding(5)
                        .width(Length::Fill),
                    
                    // Confirm button
                    button(
                        text("✓").style(|_: &Theme| text::Style {
                            color: Some(GREEN_COLOR),
                            ..Default::default()
                        })
                    )
                    .padding(5)
                    .on_press(PlaylistAction::FinishEditing)
                    .style(|_theme, _| button::Style {
                        background: None,
                        text_color: GREEN_COLOR,
                        ..Default::default()
                    })
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            } else {
                // Normal mode
                row![
                    // Playlist name button
                    button(
                        row![
                            text(&playlist.name).style(|_: &Theme| text::Style {
                                color: Some(GREEN_COLOR),
                                ..Default::default()
                            }),
                            Space::with_width(5),
                            text(format!("({} tracks)", playlist.tracks.len())).size(12).style(|_: &Theme| text::Style {
                                color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                                ..Default::default()
                            })
                        ]
                    )
                    .padding(5)
                    .width(Length::Fill)
                    .on_press(PlaylistAction::Select(id))
                    .style(|_theme, _| button::Style {
                        background: None,
                        text_color: GREEN_COLOR,
                        ..Default::default()
                    }),
                    
                    // Delete button (×) - only visible on hover
                    if is_hovered {
                        button(
                            text("×").style(|_: &Theme| text::Style {
                                color: Some(GREEN_COLOR),
                                ..Default::default()
                            })
                        )
                        .padding(5)
                        .on_press(PlaylistAction::Delete(id))
                        .style(|_theme, _| button::Style {
                            background: None,
                            text_color: GREEN_COLOR,
                            ..Default::default()
                        })
                    } else {
                        button(text("")).width(Length::Fixed(30.0))
                    }
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            };
            
            // Fix the closure lifetime issue with a direct style object
            let bg_color = if is_selected {
                Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.15)))
            } else {
                None
            };
            
            container(row_content)
                .width(Length::Fill)
                .padding(2)
                .style(move |_: &Theme| container::Style {
                    background: bg_color,
                    text_color: Some(GREEN_COLOR),
                    ..Default::default()
                })
                .into()
        })
        .collect::<Vec<Element<'_, PlaylistAction>>>()
    )
    .spacing(2)
    .width(Length::Fill);

    // Instead of .on_hover, we'll handle this in the subscription function of the app

    column![
        header,
        add_button,
        Space::with_height(10),
        scrollable(playlist_rows).height(Length::Fill),
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .into()
}