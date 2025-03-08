use iced::{Element, widget::{text, button, container, Column, Row}, Length, Alignment, theme};
use core::{PlaylistState, Action, PlaylistAction};
use crate::ui::styles::AppStyle;

pub fn view<'a>(playlists: &'a PlaylistState, style: &AppStyle) -> Element<'a, Action> {
    // Create a header
    let header = text::Text::new("Playlists")
        .size(20)
        .style(theme::Text::Color(style.colors.text_primary))
        .width(Length::Fill);
    
    // Build content for playlists
    let mut content = Column::new()
        .spacing(10)
        .padding(20)
        .push(header);
    
    // Add new playlist button
    let new_button = button::Button::new(
        text::Text::new("New Playlist")
            .style(theme::Text::Color(style.colors.button_text))
    )
    .on_press(Action::Playlist(PlaylistAction::Create("New Playlist".to_string())))
    .style(crate::ui::styles::button_style(style));
    
    content = content.push(new_button);
    
    // Display existing playlists
    if playlists.playlists.is_empty() {
        content = content.push(
            text::Text::new("No playlists yet. Create one!")
                .style(theme::Text::Color(style.colors.text_secondary))
                .width(Length::Fill)
        );
    } else {
        for (idx, playlist) in playlists.playlists.iter().enumerate() {
            let is_selected = playlists.selected.map_or(false, |sel| sel == idx);
            
            // Handle selected playlist highlighting
            let background_color = if is_selected {
                style.colors.highlight
            } else {
                style.colors.playlist_background
            };
            
            let playlist_name = text::Text::new(&playlist.name)
                .style(if is_selected {
                    theme::Text::Color(style.colors.button_text)
                } else {
                    theme::Text::Color(style.colors.text_primary)
                })
                .width(Length::Fill);
            
            let track_count = text::Text::new(format!("{} tracks", playlist.tracks.len()))
                .style(theme::Text::Color(style.colors.text_secondary));
            
            let playlist_row = Row::new()
                .spacing(10)
                .align_items(Alignment::Center)
                .push(playlist_name)
                .push(track_count);
            
            let playlist_container = container::Container::new(playlist_row)
                .width(Length::Fill)
                .padding(10)
                .style(crate::ui::styles::container_style(background_color));
            
            let playlist_button = button::Button::new(playlist_container)
                .width(Length::Fill)
                .on_press(Action::Playlist(PlaylistAction::Select(playlist.id)))
                .style(theme::Button::Text); // Use text button to remove button styling
            
            content = content.push(playlist_button);
        }
    }
    
    container::Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::ui::styles::container_style(style.colors.playlist_background))
        .into()
}