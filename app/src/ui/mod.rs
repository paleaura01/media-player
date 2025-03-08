pub mod styles;
pub mod player_view;
pub mod playlist_view;
pub mod library_view;

use iced::{Element, widget::{Container, Column, Row}};
use iced::Length;
use core::{PlayerState, PlaylistState, LibraryState, Action};

pub fn render<'a>(
    player: &'a PlayerState,
    playlists: &'a PlaylistState,
    library: &'a LibraryState
) -> Element<'a, Action> {
    // Get our style definitions - this is what makes hot reloading work
    let style = styles::default_style();
    
    // Add a version indicator to test hot-reloading
    let version_text = styles::small_text(&format!("UI Version: {}", styles::UI_VERSION));
    
    let selected_playlist_id = playlists.selected
        .and_then(|idx| playlists.playlists.get(idx))
        .map(|p| p.id);
    
    let player_view = player_view::view(player, &style);
    let playlist_view = playlist_view::view(playlists, &style);
    let library_view = library_view::view(library, selected_playlist_id, &style);
    
    let main_content = Row::new()
        .push(Container::new(playlist_view)
            .width(Length::FillPortion(20))
            .height(Length::Fill)
            .style(styles::container_style(style.colors.playlist_background)))
        .push(Container::new(library_view)
            .width(Length::FillPortion(80))
            .height(Length::Fill)
            .style(styles::container_style(style.colors.library_background)))
        .height(Length::Fill);
    
    Column::new()
        .push(version_text)
        .push(Container::new(player_view)
            .width(Length::Fill)
            .style(styles::container_style(style.colors.player_background)))
        .push(main_content)
        .into()
}