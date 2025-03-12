use iced::{Element, widget::text, Theme, Renderer};
use core::{LibraryState, Action};
use crate::ui::styles::AppStyle;

pub fn view<'a>(
    _library: &'a LibraryState, 
    _selected_playlist_id: Option<u32>, 
    _style: &AppStyle
) -> Element<'a, Action, Theme, Renderer> {
    // Replace hot reloading reference with normal text
    let display_text = "Library View FAGSSSSSSSSSSSSSSSSSSSsSSSS";
    
    // Simple styling that will work with iced 0.13.1
    text::Text::new(display_text)
        .size(24)
        .into()
}