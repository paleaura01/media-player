use iced::widget::{column, container, text};
use iced::{Element, Length};
use core::library::LibraryState;
use crate::ui::theme::library_container_style;

pub fn view(_library: &LibraryState) -> Element<()> {
    let content = column![
        text("Library View").size(24).style(|_| iced::widget::text::Style {
            color: Some(iced::Color::from_rgb(0.0, 1.0, 0.0)),
            ..Default::default()
        }),
        // Add more library UI elements here
    ]
    .spacing(10)
    .padding(15);
    
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(library_container_style())
        .into()
}