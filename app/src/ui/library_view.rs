use iced::{Element, widget::{text, container, Column, Row, button}, Length, Alignment, theme};
use core::{LibraryState, Action, PlaylistAction, PlayerAction, LibraryAction};
use crate::ui::styles::AppStyle;

pub fn view<'a>(
    library: &'a LibraryState, 
    selected_playlist_id: Option<u32>, 
    style: &AppStyle
) -> Element<'a, Action> {
    // Create main content column with header directly
    let mut content = Column::new()
        .spacing(15)
        .padding(20)
        .push(
            text::Text::new("Library")
                .size(24)
                .style(theme::Text::Color(style.colors.text_primary))
        );
    
    // Add drag and drop area - avoid storing text in variable
    let drag_area = container::Container::new(
        text::Text::new("Drag and drop audio files here")
            .style(theme::Text::Color(style.colors.text_secondary))
    )
    .width(Length::Fill)
    .padding(30)
    .style(crate::ui::styles::container_style(style.colors.player_background));
    
    content = content.push(drag_area);
    
    // Display library tracks - still avoiding intermediate variables
    if library.tracks.is_empty() {
        content = content.push(
            text::Text::new("No tracks in library. Import or drag files to add.")
                .style(theme::Text::Color(style.colors.text_secondary))
                .width(Length::Fill)
        );
    } else {
        // Add column headers directly
        let headers = Row::new()
            .spacing(10)
            .padding(5)
            .push(text::Text::new("Title").width(Length::FillPortion(3)).style(theme::Text::Color(style.colors.text_primary)))
            .push(text::Text::new("Artist").width(Length::FillPortion(2)).style(theme::Text::Color(style.colors.text_primary)))
            .push(text::Text::new("Album").width(Length::FillPortion(2)).style(theme::Text::Color(style.colors.text_primary)));
        
        content = content.push(headers);
        
        // Add track rows
        let mut tracks_column = Column::new().spacing(2);
        
        for track in &library.tracks {
            let title = track.title.as_deref().unwrap_or("Unknown");
            let artist = track.artist.as_deref().unwrap_or("-");
            let album = track.album.as_deref().unwrap_or("-");
            
            let track_path = track.path.clone();
            
            // Build the row directly without intermediate variables
            let track_row = Row::new()
                .spacing(10)
                .padding(5)
                .push(text::Text::new(title).width(Length::FillPortion(3)).style(theme::Text::Color(style.colors.text_primary)))
                .push(text::Text::new(artist).width(Length::FillPortion(2)).style(theme::Text::Color(style.colors.text_secondary)))
                .push(text::Text::new(album).width(Length::FillPortion(2)).style(theme::Text::Color(style.colors.text_secondary)));
            
            // Combine row with buttons
            let track_button = Row::new()
                .push(track_row)
                .push(
                    button::Button::new(
                        text::Text::new("+").style(theme::Text::Color(style.colors.button_text))
                    )
                    .on_press(Action::Player(PlayerAction::Play(track_path.clone())))
                    .style(crate::ui::styles::button_style(style))
                )
                .align_items(Alignment::Center);
            
            // Add to playlist button (if a playlist is selected)
            let track_container = if let Some(playlist_id) = selected_playlist_id {
                let add_track = track.clone();
                Row::new()
                    .push(track_button)
                    .push(
                        button::Button::new(
                            text::Text::new("+ Playlist").style(theme::Text::Color(style.colors.text_secondary))
                        )
                        .on_press(Action::Playlist(PlaylistAction::AddTrack(playlist_id, add_track)))
                    )
                    .align_items(Alignment::Center)
            } else {
                Row::new().push(track_button)
            };
            
            tracks_column = tracks_column.push(track_container);
        }
        
        content = content.push(tracks_column);
    }
    
    // Add import button directly
    content = content.push(
        button::Button::new(
            text::Text::new("Import").style(theme::Text::Color(style.colors.button_text))
        )
        .on_press(Action::Library(LibraryAction::StartScan))
        .style(crate::ui::styles::button_style(style))
    );
    
    // Main container
    container::Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::ui::styles::container_style(style.colors.library_background))
        .into()
}