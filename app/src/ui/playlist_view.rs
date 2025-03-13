use iced::widget::{column, container, text};
use iced::{Element, Length};
use core::playlist::PlaylistState;
// Removed unused import: green_text

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
        .style(|_| iced::widget::container::Style {
            border: iced::Border {
                color: iced::Color::from_rgb(0.0, 0.5, 0.0),
                width: 1.0,
                radius: 5.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}