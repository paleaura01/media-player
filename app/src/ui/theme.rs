use iced::widget::{container, button, text, progress_bar};
use iced::{Background, Color, Border, Shadow, Vector};

// Define a better color palette with lighter shades
pub const BLACK_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
// Make player/playlist background lighter, but still darker than library
pub const DARK_BG_COLOR: Color = Color { r: 0.10, g: 0.10, b: 0.10, a: 1.0 }; 
// Keep library view slightly lighter than player/playlist
pub const MEDIUM_BG_COLOR: Color = Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 };
pub const GREEN_COLOR: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
pub const DARK_GREEN_COLOR: Color = Color { r: 0.0, g: 0.5, b: 0.0, a: 1.0 };
pub const DARKER_GREEN_COLOR: Color = Color { r: 0.0, g: 0.3, b: 0.0, a: 1.0 };

// Add a function to provide a dark theme
pub fn dark_theme() -> iced::Theme {
    iced::Theme::Dark
}

// Borderless container for player and playlist
pub fn borderless_dark_container_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(Background::Color(DARK_BG_COLOR)),
        border: Border::default(), // No border
        text_color: Some(GREEN_COLOR),
        ..Default::default()
    }
}

// Library view with border to separate it from playlist
pub fn library_container_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(Background::Color(MEDIUM_BG_COLOR)),
        border: Border {
            color: DARK_GREEN_COLOR,
            width: 1.0,
            radius: 0.0.into(),
            ..Default::default()
        },
        text_color: Some(GREEN_COLOR),
        ..Default::default()
    }
}

/// Returns a progress bar with dark fill on a dark background.
pub fn green_progress_bar<'a>(value: f32, max: f32) -> progress_bar::ProgressBar<'a> {
    progress_bar(value..=max, 200.0)
        .style(|_theme| progress_bar::Style {
             background: Background::Color(DARK_BG_COLOR),
             bar: Background::Color(DARK_BG_COLOR),
             border: Border {
                color: DARK_GREEN_COLOR,
                width: 1.0,
                radius: 0.0.into(),
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
                Background::Color(DARKER_GREEN_COLOR),
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
                radius: 0.0.into(),
                ..Default::default()
            },
            text_color: BLACK_COLOR,
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

