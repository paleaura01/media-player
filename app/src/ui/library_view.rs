use iced::widget::column;
use iced::Element;
use core::library::LibraryState;
use crate::ui::theme::green_text;

pub fn view(_library: &LibraryState) -> Element<()> {
    column![
        green_text("Library View"),
    ]
    .spacing(10)
    .into()
}
