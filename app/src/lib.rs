// app/src/lib.rs
// This file is the entry point for the dynamic library.

// Import the UI module
#[path = "ui/mod.rs"]
pub mod ui;

// Re-export the render function for hot reloading
#[no_mangle]
pub extern "C" fn render(
    player: &core::PlayerState,
    playlists: &core::PlaylistState,
    library: &core::LibraryState,
) -> ui::UiElement {
    ui::render(player, playlists, library)
}