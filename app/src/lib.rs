// Export your UI and render functions for hot reloading
pub mod ui;

// This function is exported for hot reloading and called by the main app
#[no_mangle]
pub fn render<'a>(
    player: &'a core::PlayerState,
    playlists: &'a core::PlaylistState,
    library: &'a core::LibraryState
) -> iced::Element<'a, core::Action> {
    // Force the UI to rebuild from scratch each time this function is called
    // This ensures we're getting the latest module version
    ui::render(player, playlists, library)
}