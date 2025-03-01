use iced::{Element, widget::{Button, Text, Column, Row, Scrollable}};
use iced::Length;
use core::{Action, LibraryState, LibraryAction, PlaylistAction, Track};

pub fn view(library: &LibraryState, selected_playlist_id: Option<u32>) -> Element<'static, Action> {
    let title = Text::new("Library")
        .size(20)
        .width(Length::Fill)
        .horizontal_alignment(iced::alignment::Horizontal::Center);

    // Create scan button row
    let scan_button = Button::new(Text::new("Scan for Music"))
        .on_press(Action::Library(LibraryAction::StartScan))
        .padding(10);

    let header_row = Row::new()
        .push(title)
        .push(scan_button)
        .spacing(10)
        .align_items(iced::Alignment::Center);

    // Create table headers
    let headers = Row::new()
        .push(Text::new("Title").width(Length::FillPortion(3)))
        .push(Text::new("Artist").width(Length::FillPortion(2)))
        .push(Text::new("Album").width(Length::FillPortion(2)))
        .push(Text::new("Actions").width(Length::FillPortion(1)))
        .padding(5)
        .spacing(10);

    // Create list of tracks
    let mut track_items = Vec::new();
    track_items.push(headers.into());

    for track in &library.tracks {
        let title = track.title.clone().unwrap_or_else(|| 
            std::path::Path::new(&track.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown")
                .to_string()
        );

        let add_button = if let Some(playlist_id) = selected_playlist_id {
            Button::new(Text::new("Add"))
                .on_press(Action::Playlist(PlaylistAction::AddTrack(
                    playlist_id,
                    track.clone()
                )))
                .padding(5)
        } else {
            Button::new(Text::new("Select Playlist"))
                .padding(5)
        };

        let track_row = Row::new()
            .push(Text::new(title).width(Length::FillPortion(3)))
            .push(Text::new(track.artist.clone().unwrap_or_else(|| "-".to_string())).width(Length::FillPortion(2)))
            .push(Text::new(track.album.clone().unwrap_or_else(|| "-".to_string())).width(Length::FillPortion(2)))
            .push(add_button.width(Length::FillPortion(1)))
            .padding(5)
            .spacing(10);

        track_items.push(track_row.into());
    }

    // Empty library message
    if library.tracks.is_empty() {
        track_items.push(
            Text::new("No tracks in library. Scan for music to get started.")
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .into()
        );
    }

    let tracks_scroll = Scrollable::new(
        Column::with_children(track_items)
            .spacing(2)
    )
    .height(Length::Fill)
    .width(Length::Fill);

    Column::new()
        .push(header_row)
        .push(tracks_scroll)
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}