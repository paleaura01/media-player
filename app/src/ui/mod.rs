pub mod styles;
pub mod player_view;
pub mod playlist_view;
pub mod library_view;

use iced::{Element, widget::{Container, Column, Row}, Length};
use core::{PlayerState, PlaylistState, LibraryState, Action};

pub fn render<'a>(
    player: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState
) -> Element<'a, Action> {
    // Get our style definitions
    let style = styles::default_style();
    
    // Remove version indicator text
    
    let selected_playlist_id = playlists.selected
        .and_then(|idx| playlists.playlists.get(idx))
        .map(|p| p.id);
    
    let player_view = player_view::view(player, &style);
    let playlist_view = crate::ui::playlist_view::view(playlists, &style);
    let library_view = crate::ui::library_view::view(library, selected_playlist_id, &style);
    
    let main_content = Row::new()
        .push(
            Container::new(playlist_view)
                .width(Length::FillPortion(25))
                .style(move |_| styles::container_style(style.colors.playlist_background))
        )
        .push(
            Container::new(library_view)
                .width(Length::FillPortion(75))
                .style(move |_| styles::container_style(style.colors.library_background))
        )
        .height(Length::Fill);
    
    Column::new()
        // Remove version_text from here
        .push(
            Container::new(player_view)
                .style(move |_| styles::container_style(style.colors.player_background))
        )
        .push(main_content)
        .into()
}