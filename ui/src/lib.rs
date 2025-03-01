mod player_view;
mod playlist_view;
mod library_view;

use iced::{Element, widget::{Container, Column, Row}};
use iced::Length;
use core::{PlayerState, PlaylistState, LibraryState, Action};

#[no_mangle]
pub fn render(
    player: &PlayerState,
    playlists: &PlaylistState,
    library: &LibraryState
) -> Element<'static, Action> {
    // Get selected playlist ID for library view
    let selected_playlist_id = playlists.selected
        .map(|idx| playlists.playlists.get(idx))
        .flatten()
        .map(|p| p.id);

    // Create the player section at the top
    let player_view = player_view::view(player);
    
    // Create the playlist section on the left (25% width)
    let playlist_view = playlist_view::view(playlists);
    
    // Create the library section on the right (75% width)
    let library_view = library_view::view(library, selected_playlist_id);
    
    // Layout with player at top, playlist on left, library on right
    let main_content = Row::new()
        .push(playlist_view.width(Length::FillPortion(25)))
        .push(library_view.width(Length::FillPortion(75)))
        .height(Length::Fill);
    
    Column::new()
        .push(player_view)
        .push(main_content)
        .into()
}