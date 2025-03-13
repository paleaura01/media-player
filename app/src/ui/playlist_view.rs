// app/src/ui/playlist_view.rs
use iced::widget::{button, column, container, row, text, scrollable, Space, text_input, image};
use iced::{Alignment, Element, Length, Theme};
use core::playlist::PlaylistState;
use crate::ui::theme::GREEN_COLOR;
use crate::states::playlist_state::PlaylistViewState;

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

// Function to load an icon with proper logging
fn load_icon(name: &str) -> image::Handle {
    let base_path = std::env::current_dir().unwrap_or_default();
    let icon_path = base_path.join("app").join("assets").join("icons").join(name);
    
    // Log the full path for debugging
    println!("Loading icon from: {}", icon_path.display());
    
    image::Handle::from_path(icon_path)
}

// Enhanced view with editing state and hover delete functionality
pub fn view_with_state<'a>(playlist_state: &'a PlaylistState, view_state: &'a PlaylistViewState) -> Element<'a, PlaylistAction> {
    let header = text("Playlists")
        .size(20)
        .style(|_: &Theme| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        });
    
    // Load icons with better handling
    let plus_icon = load_icon("ph--file-plus-thin.svg");
    let x_icon = load_icon("ph--x-square-thin.svg");
    
    let add_button = button(
        row![
            image(plus_icon)
                .width(16)
                .height(16),
            Space::with_width(5),
            text("Add Playlist").style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            })
        ]
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
                    // Playlist name button (without track count)
                    button(
                        text(&playlist.name).style(|_: &Theme| text::Style {
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
                    
                    // Delete button (×) - only visible on hover or selected
                    if is_hovered {
                        button(
                            image(x_icon.clone())
                                .width(16)
                                .height(16)
                        )
                        .padding(5)
                        .on_press(PlaylistAction::Delete(id))
                        .style(|_theme, _| button::Style {
                            background: None,
                            ..Default::default()
                        })
                    } else {
                        button(Space::new(Length::Fixed(16.0), Length::Fixed(16.0)))
                            .padding(5)
                            .style(|_theme, _| button::Style {
                                background: None,
                                ..Default::default()
                            })
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
            
            // Use mouse over detection for hover
            let hover_container = container(row_content)
                .width(Length::Fill)
                .padding(2)
                .style(move |_: &Theme| container::Style {
                    background: bg_color,
                    text_color: Some(GREEN_COLOR),
                    ..Default::default()
                });
            
            // Wrap in row for hover detection
            row![
                hover_container,
            ]
            .width(Length::Fill)
            .into()
        })
        .collect::<Vec<Element<'_, PlaylistAction>>>()
    )
    .spacing(2)
    .width(Length::Fill);

    // Main playlist view
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