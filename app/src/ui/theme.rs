use iced::widget::{container, text, progress_bar};
use iced::{Background, Color, Border};

// Keep only the colors that are actually used
// Make player/playlist background lighter, but still darker than library
pub const DARK_BG_COLOR: Color = Color { r: 0.10, g: 0.10, b: 0.10, a: 1.0 }; 
// Keep library view slightly lighter than player/playlist
pub const MEDIUM_BG_COLOR: Color = Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 };
pub const GREEN_COLOR: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
pub const DARK_GREEN_COLOR: Color = Color { r: 0.0, g: 0.5, b: 0.0, a: 1.0 };
// Renamed to better reflect its actual use - a semi-transparent background for controls
pub const PROGRESS_BG_COLOR: Color = Color { r: 0.1, g: 0.1, b: 0.1, a: 0.5 };

// Add a function to provide a dark theme
pub fn dark_theme() -> iced::Theme {
    iced::Theme::Dark
}

// Player container style with border
pub fn player_container_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(Background::Color(DARK_BG_COLOR)),
        border: Border {
            color: DARK_GREEN_COLOR,
            width: 1.0,
            radius: 0.0.into(),  // Square corners
            ..Default::default()
        },
        text_color: Some(GREEN_COLOR),
        ..Default::default()
    }
}

// Playlist panel style with border
pub fn playlist_container_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(Background::Color(DARK_BG_COLOR)),
        border: Border {
            color: DARK_GREEN_COLOR,
            width: 1.0,
            radius: 0.0.into(),  // Set this to 0.0 for square corners at edges
            ..Default::default()
        },
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

// Now playing container style with border
pub fn now_playing_container_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(Background::Color(DARK_BG_COLOR)),
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

/// Returns a stylized green progress bar with transparent background and green fill
pub fn green_progress_bar<'a>(value: f32) -> progress_bar::ProgressBar<'a> {
    progress_bar(0.0..=1.0, value)
        .style(|_theme| progress_bar::Style {
             // Now using our named constant instead of an inline color definition
             background: Background::Color(PROGRESS_BG_COLOR),
             bar: Background::Color(GREEN_COLOR),
             border: Border {
                color: DARK_GREEN_COLOR,
                width: 1.0,
                radius: 2.0.into(), // Slight rounding for better look
                ..Default::default()
             },
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