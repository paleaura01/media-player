// app/src/ui/library_view.rs
use iced::{Element, widget::text};
use core::{LibraryState, Action};
use crate::ui::styles::AppStyle;

pub fn view(_library: &LibraryState, _selected_playlist_id: Option<u32>, _style: &AppStyle) -> Element<'static, Action> {
    // This text will change when hot reloading works
    let display_text = "LIBRARY VIEW - HOT";
    
    // Simple styling that will work with iced 0.13.1
    text::Text::new(display_text)
        .size(24)
        .into()
}
