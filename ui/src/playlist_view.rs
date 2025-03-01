use iced::{Element, widget::{Button, Text, Column, Scrollable, Container}};
use iced::Length;
use core::{Action, PlaylistState, PlaylistAction, Track};

pub fn view(playlists: &PlaylistState) -> Element<'static, Action> {
    // Create "New Playlist" button
    let new_button = Button::new(Text::new("New Playlist"))
        .on_press(Action::Playlist(PlaylistAction::Create("New Playlist".to_string())))
        .width(Length::Fill)
        .padding(10);
    
    // Create scrollable list of playlists
    let mut playlist_items = Vec::new();
    
    // Add each playlist as a button
    for playlist in &playlists.playlists {
        let is_selected = playlists.selected
            .map(|idx| playlists.playlists.get(idx))
            .flatten()
            .map(|p| p.id == playlist.id)
            .unwrap_or(false);
        
        let playlist_button = Button::new(Text::new(&playlist.name))
            .on_press(Action::Playlist(PlaylistAction::Select(playlist.id)))
            .padding(10)
            .width(Length::Fill)
            .style(if is_selected {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            });
        
        playlist_items.push(playlist_button.into());
    }
    
    // If there are no playlists, show a message
    if playlist_items.is_empty() {
        playlist_items.push(
            Text::new("No playlists. Create one to get started!")
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .into()
        );
    }
    
    // Combine into scrollable view
    let playlists_scroll = Scrollable::new(
        Column::with_children(playlist_items)
            .spacing(5)
    )
    .height(Length::Fill)
    .width(Length::Fill);
    
    // Full layout with button at top, scrollable list below
    Column::new()
        .push(Text::new("Playlists").size(20))
        .push(new_button)
        .push(playlists_scroll)
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}