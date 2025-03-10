use iced::{Element, widget::text, Theme, Renderer};
use core::{LibraryState, Action};
use crate::ui::styles::AppStyle;

pub fn view<'a>(
    _library: &'a LibraryState, 
    _selected_playlist_id: Option<u32>, 
    _style: &AppStyle
) -> Element<'a, Action, Theme, Renderer> {
    // This text will change when hot reloading works
    let display_text = "LIBRARY VIEW - HOT 2 2RELOADING IS WORKING!!!!!";
    
    // Simple styling that will work with iced 0.13.1
    text::Text::new(display_text)
        .size(24)
        .into()
}
