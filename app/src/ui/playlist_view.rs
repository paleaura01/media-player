use iced::{Element, widget::{text, button, container, Column, Row, text_input, Container}, Length, Alignment};
use iced::theme;
use core::{PlaylistState, Action, PlaylistAction};
use crate::ui::styles::AppStyle;

pub fn view<'a>(playlists: &'a PlaylistState, style: &AppStyle) -> Element<'a, Action> {
    // Create a header with double-click instruction
    let header = text::Text::new("Playlists - Double-click to edit")
        .size(20)
        .width(Length::Fill);
    
    // Build content for playlists
    let mut content = Column::new()
        .spacing(10)
        .padding(20)
        .push(header);
    
    // Add a create button
    let create_button = button::Button::new(
        text::Text::new("Create New Playlist")
    )
    .on_press(Action::Playlist(PlaylistAction::Create("New Playlist".to_string())));
    
    content = content.push(create_button);
    
    // Display existing playlists
    if playlists.playlists.is_empty() {
        content = content.push(
            text::Text::new("No playlists yet. Create one!")
                .width(Length::Fill)
        );
    } else {
        for playlist in &playlists.playlists {
            // Check if this playlist is being edited
            if playlists.editing_id == Some(playlist.id) {
                // Simple editing interface - just a text input
                let text_input = text_input::TextInput::new(
                    "Enter playlist name",
                    &playlists.editing_text
                )
                .on_input(|text| Action::Playlist(PlaylistAction::UpdateEditingText(text)))
                .on_submit(Action::Playlist(PlaylistAction::SaveEdit));
                
                content = content.push(text_input.width(Length::Fill));
            } else {
                // Check if this playlist is selected
                let is_selected = playlists.selected
                    .and_then(|idx| playlists.playlists.get(idx))
                    .map(|p| p.id) == Some(playlist.id);
                
                // Create the row with playlist info
                let playlist_row = Row::new()
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .push(
                        text::Text::new(&playlist.name)
                            .width(Length::Fill)
                    )
                    .push(
                        text::Text::new(format!("{} tracks", playlist.tracks.len()))
                    );
                
                // Create a clickable container for double-click detection
                let clickable_row = button::Button::new(Container::new(playlist_row))
                    .on_press(Action::Playlist(PlaylistAction::Click(playlist.id)))
                    .width(Length::Fill)
                    .style(if is_selected {
                        // Use highlight style for selected playlist
                        crate::ui::styles::selected_button_style(style)
                    } else {
                        // Use regular style for other playlists
                        crate::ui::styles::button_style(style)
                    });
                
                content = content.push(clickable_row);
            }
        }
    }
    
    container::Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}