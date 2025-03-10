// app/src/ui/mod.rs
pub mod styles;
pub mod player_view;
pub mod playlist_view;
pub mod library_view;

use iced::{Element, widget::{Container, Column, Row}, Length};
use core::{PlayerState, PlaylistState, LibraryState, Action};

// This must be repr(C) for FFI compatibility
#[repr(C)]
pub struct UiElement(pub Element<'static, Action>);

pub fn render(
    player: &PlayerState,
    playlists: &PlaylistState,
    library: &LibraryState,
) -> UiElement {
    let style = styles::default_style();
    
    let version_text = styles::small_text(&format!("UI Version: {}", styles::UI_VERSION));
    
    let selected_playlist_id = playlists.selected
        .and_then(|idx| playlists.playlists.get(idx))
        .map(|p| p.id);
    
    let player_view = player_view::view(player, &style);
    let playlist_view = playlist_view::view(playlists, &style);
    let library_view = library_view::view(library, selected_playlist_id, &style);
    
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
    
    let element = Column::new()
        .push(version_text)
        .push(
            Container::new(player_view)
                .style(move |_| styles::container_style(style.colors.player_background))
        )
        .push(main_content)
        .into();
    
    UiElement(element)
}