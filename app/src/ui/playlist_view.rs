use iced::{Element, widget::{text, button, container, Column, Row}, Length, Alignment};
use core::{PlaylistState, Action, PlaylistAction};
use crate::ui::styles::AppStyle;

pub fn view<'a>(playlists: &'a PlaylistState, _style: &AppStyle) -> Element<'a, Action> {
    // Create a header with version info for testing hot reloading
    let header = text::Text::new("Playlist View - HOT RELOADED!")
        .size(20)
        .width(Length::Fill);
    
    // Build content for playlists
    let mut content = Column::new()
        .spacing(10)
        .padding(20)
        .push(header);
    
    // Add a test button
    let test_button = button::Button::new(
        text::Text::new("Create Test Playlist")
    )
    .on_press(Action::Playlist(PlaylistAction::Create("Test Playlist".to_string())));
    
    content = content.push(test_button);
    
    // Display existing playlists
    if playlists.playlists.is_empty() {
        content = content.push(
            text::Text::new("No playlists yet. Create one!")
                .width(Length::Fill)
        );
    } else {
        for playlist in &playlists.playlists {
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
            
            content = content.push(playlist_row);
        }
    }
    
    container::Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}