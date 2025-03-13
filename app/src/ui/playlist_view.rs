use iced::widget::column;
use iced::Element;
use core::playlist::PlaylistState;
use crate::ui::theme::green_text;

pub fn view(_playlist: &PlaylistState) -> Element<()> {
    column![
        green_text("Playlist View"),
    ]
    .spacing(10)
    .into()
}
