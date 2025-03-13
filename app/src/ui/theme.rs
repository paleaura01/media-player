use iced::widget::{container, button, text, progress_bar};
use iced::{Background, Color, Border, Shadow, Vector};  // Import Shadow from iced directly

pub const BLACK_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
pub const GREEN_COLOR: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
pub const DARK_GREEN_COLOR: Color = Color { r: 0.0, g: 0.5, b: 0.0, a: 1.0 };

// Add a function to provide a dark theme
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
pub fn green_progress_bar<'a>(value: f32, max: f32) -> progress_bar::ProgressBar<'a> {
    progress_bar(value..=max, 200.0)
        .style(|_theme| progress_bar::Style {
             background: Background::Color(Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
             bar: Background::Color(GREEN_COLOR),
             border: Border {
                color: DARK_GREEN_COLOR,
                width: 1.0,
                radius: 5.0.into(),
                ..Default::default()
             },
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
    .style(|_theme, status| {
        let (background, border_color) = match status {
            button::Status::Pressed => (
                Background::Color(DARK_GREEN_COLOR),
                DARK_GREEN_COLOR
            ),
            _ => (
                Background::Color(GREEN_COLOR),
                DARK_GREEN_COLOR
            ),
        };
        
        button::Style {
            background: Some(background),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 5.0.into(),
                ..Default::default()
            },
            text_color: BLACK_COLOR,
            // Use the Shadow struct directly from iced
            shadow: Shadow {
                offset: Vector::new(1.0, 1.0),
                blur_radius: 2.0,
                color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.5 },
            },
            ..Default::default()
        }
    })
    .padding([5, 10])
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