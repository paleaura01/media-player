// app/src/ui/playlist_view.rs
use iced::widget::{button, column, container, row, text, scrollable, Space, text_input};
use iced::{Alignment, Element, Length};
use core::playlist::PlaylistState;
use crate::ui::theme::GREEN_COLOR;
use crate::states::playlist_state::PlaylistViewState; // Updated import path

#[derive(Debug, Clone)]
pub enum PlaylistAction {
    Create(String),
    Delete(u32),
    Rename(u32, String),
    Select(u32),
    StartEditing(u32, String),
    EditingName(String),
    FinishEditing,
    None,
}

// Simple view without editing state
pub fn view<'a>(playlist_state: &'a PlaylistState) -> Element<'a, PlaylistAction> {
    let header = text("Playlist View")
        .size(24)
        .style(|_| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        });
    
    let add_button = button(
        text("+ Add Playlist").style(|_| text::Style {
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

    // Fixed type annotation issue by using Vec<Element<'_, PlaylistAction>>
    let playlist_rows = column(
        playlist_state.playlists.iter().enumerate().map(|(idx, playlist)| {
            let id = idx as u32;
            
            let playlist_row = row![
                // Playlist name button
                button(
                    text(&playlist.name).style(|_| text::Style {
                        color: Some(GREEN_COLOR),
                        ..Default::default()
                    })
                )
                .padding(5)
                .width(Length::Fill)
                .on_press(PlaylistAction::Select(id))
                .style(|_theme, _| button::Style {
                    background: None,
                    text_color: GREEN_COLOR,
                    ..Default::default()
                }),
                
                // Delete button (×)
                button(
                    text("×").style(|_| text::Style {
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
            ]
            .spacing(5)
            .align_y(Alignment::Center);
            
            container(playlist_row)
                .width(Length::Fill)
                .style(|_| container::Style {
                    text_color: Some(GREEN_COLOR),
                    ..Default::default()
                })
                .into()
        })
        .collect::<Vec<Element<'_, PlaylistAction>>>() // Added type annotation
    )
    .spacing(2)
    .width(Length::Fill);

    let content = column![
        header,
        add_button,
        Space::with_height(10),
        scrollable(playlist_rows).height(Length::Fill),
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill);
    
    content.into()
}

// Enhanced view with editing state
pub fn view_with_state<'a>(playlist_state: &'a PlaylistState, view_state: &'a PlaylistViewState) -> Element<'a, PlaylistAction> {
    let header = text("Playlist View")
        .size(24)
        .style(|_| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        });
    
    let add_button = button(
        text("+ Add Playlist").style(|_| text::Style {
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

    // Fixed type annotation issue
    let playlist_rows = column(
        playlist_state.playlists.iter().enumerate().map(|(idx, playlist)| {
            let id = idx as u32;
            
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
                        text("✓").style(|_| text::Style {
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
                        text(&playlist.name).style(|_| text::Style {
                            color: Some(GREEN_COLOR),
                            ..Default::default()
                        })
                    )
                    .padding(5)
                    .width(Length::Fill)
                    .on_press(PlaylistAction::Select(id))
                    .style(|_theme, _| button::Style {
                        background: None,
                        text_color: GREEN_COLOR,
                        ..Default::default()
                    }),
                    
                    // Delete button (×)
                    button(
                        text("×").style(|_| text::Style {
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
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            };
            
            container(row_content)
                .width(Length::Fill)
                .style(|_| container::Style {
                    text_color: Some(GREEN_COLOR),
                    ..Default::default()
                })
                .into()
        })
        .collect::<Vec<Element<'_, PlaylistAction>>>() // Added type annotation
    )
    .spacing(2)
    .width(Length::Fill);

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