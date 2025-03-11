use iced::{Color, Background, Vector, Border, Shadow, Theme};
use iced::widget::container::Style as ContainerStyle;
use iced::widget::button::Style as ButtonStyle;

// Remove UI_VERSION constants
// pub const UI_VERSION: &str = "3.1.0";
// #[no_mangle]
// pub static UI_VERSION_INT: u64 = 5;

#[derive(Debug, Clone)]
pub struct AppStyle {
    pub colors: ColorTheme,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ColorTheme {
    pub player_background: Color,
    pub playlist_background: Color,
    pub library_background: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub button_background: Color,
    pub button_hover: Color,
    pub button_text: Color,
    pub highlight: Color,
    pub border: Color,
}

pub fn default_style() -> AppStyle {
    AppStyle {
        colors: ColorTheme {
            player_background: Color::from_rgb(0.12, 0.12, 0.12),
            playlist_background: Color::from_rgb(0.16, 0.16, 0.18),
            library_background: Color::from_rgb(0.16, 0.16, 0.18),
            text_primary: Color::from_rgb(0.9, 0.9, 0.9),
            text_secondary: Color::from_rgb(0.7, 0.7, 0.7),
            button_background: Color::from_rgb(0.2, 0.2, 0.25),
            button_hover: Color::from_rgb(0.25, 0.25, 0.3),
            button_text: Color::from_rgb(0.9, 0.9, 0.9),
            highlight: Color::from_rgb(0.35, 0.65, 0.85),
            border: Color::from_rgb(0.25, 0.25, 0.3),
        }
    }
}

/// Constructs a container style with the specified background color.
pub fn container_style(color: Color) -> ContainerStyle {
    ContainerStyle {
        background: Some(Background::Color(color)),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            offset: Vector::new(0.0, 0.0),
            blur_radius: 0.0,
            color: Color::TRANSPARENT,
        },
        text_color: None,
    }
}

#[allow(dead_code)]
/// Returns a text widget with small text styling.
pub fn small_text(content: &str) -> iced::widget::Text<'static, Theme> {
    iced::widget::Text::new(content.to_owned())
        .size(12)
        .color(Color::from_rgb(0.7, 0.7, 0.7))
}

/// Constructs a button style using colors from your AppStyle.
#[allow(dead_code)]
pub fn button_style(style: &AppStyle) -> ButtonStyle {
    ButtonStyle {
        background: Some(Background::Color(style.colors.button_background)),
        border: Border {
            width: 1.0,
            color: style.colors.border,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            offset: Vector::new(0.0, 0.0),
            blur_radius: 0.0,
            color: Color::TRANSPARENT,
        },
        text_color: style.colors.button_text,
    }
}
