use iced::widget::{column, container, text};
use iced::{Element, Length};
use core::playlist::PlaylistState;
use crate::ui::theme::playlist_container_style;

pub fn view(_playlist: &PlaylistState) -> Element<()> {
    let content = column![
        text("Playlist View").size(24).style(|_| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.0, 1.0, 0.0)),
            ..Default::default()
        }),
        // Add more playlist UI elements here
    ]
    .spacing(10)
    .padding(15);
    
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(playlist_container_style())
        .into()
}