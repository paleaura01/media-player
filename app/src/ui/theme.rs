use iced::widget::{container, button, text, progress_bar};
use iced::{Background, Color, Border}; // Import Border from iced directly

pub const BLACK_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
pub const GREEN_COLOR: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };

// Add dark theme function for application.rs
pub fn dark_theme() -> iced::Theme {
    iced::Theme::Dark
}

/// Returns a container with a black background and green text.
pub fn black_green_container<'a, M, T>(content: T) -> container::Container<'a, M>
where
    T: Into<iced::Element<'a, M>>,
{
    container(content).style(|_theme| container::Style {
        background: Some(Background::Color(BLACK_COLOR)),
        text_color: Some(GREEN_COLOR),
        ..Default::default()
    })
}

/// Returns a progress bar with green fill on a black background.
/// A fixed width of 200.0 is used.
pub fn green_progress_bar<'a>(value: f32, max: f32) -> progress_bar::ProgressBar<'a> {
    progress_bar(value..=max, 200.0).style(|_theme| progress_bar::Style {
         background: Background::Color(BLACK_COLOR),
         bar: Background::Color(GREEN_COLOR),
         border: Border::default(), // Use Border::default() instead
    })
}

/// Returns a button with a green background and black text.
pub fn green_button<'a, M>(label: &'a str, on_press: M) -> button::Button<'a, M>
where
    M: Clone,
{
    button(
        text(label).style(|_theme| text::Style {
            color: Some(BLACK_COLOR),
            ..Default::default()
        }),
    )
    .on_press(on_press)
    .style(|_theme, _status| button::Style {
        background: Some(Background::Color(GREEN_COLOR)),
        ..Default::default()
    })
}

/// Returns a text widget styled in green.
pub fn green_text<'a, S>(label: S) -> text::Text<'a>
where
    S: Into<String>,
{
    text(label.into()).style(|_theme| text::Style {
        color: Some(GREEN_COLOR),
        ..Default::default()
    })
}