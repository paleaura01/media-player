// app/src/ui/playlist_view.rs
use iced::{Element, Length, Alignment};
use iced::widget::{Text, Button, Container, Column, Row};
use core::{PlaylistState, Action, PlaylistAction};
use crate::ui::styles::AppStyle;

pub fn view(playlists: &PlaylistState, _style: &AppStyle) -> Element<'static, Action> {
    // Create a header with version info for testing hot reloading
    let header = Text::new("Playlist View - HOT RELOADED!")
        .size(20)
        .width(Length::Fill);
    
    // Build content for playlists
    let mut content = Column::new()
        .spacing(10)
        .padding(20)
        .push(header);
    
    // Add a test button
    let test_button = Button::new(Text::new("Create Test Playlist"))
        .on_press(Action::Playlist(PlaylistAction::Create("Test Playlist".to_string())));
    
    content = content.push(test_button);
    
    // Display existing playlists
    if playlists.playlists.is_empty() {
        content = content.push(
            Text::new("No playlists yet. Create one!")
                .width(Length::Fill)
        );
    } else {
        for playlist in &playlists.playlists {
            let playlist_row = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    Text::new(playlist.name.clone())
                        .width(Length::Fill)
                )
                .push(
                    Text::new(format!("{} tracks", playlist.tracks.len()))
                );
            
            content = content.push(playlist_row);
        }
    }
    
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
