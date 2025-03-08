use iced::{Color, Background, widget::{container, text}};
use iced::{Theme, theme};

// Increment this version number whenever you update UI to trigger hot-reloading
#[no_mangle]
pub static UI_VERSION_INT: u64 = 2;

// Updated version to test hot reloading
pub const UI_VERSION: &str = "2.0.0";

#[derive(Debug, Clone)]
pub struct AppStyle {
    pub colors: ColorTheme,
}

#[derive(Debug, Clone)]
pub struct ColorTheme {
    pub player_background: Color,
    pub playlist_background: Color,
    pub library_background: Color,
}

pub fn default_style() -> AppStyle {
    AppStyle {
        colors: ColorTheme {
            player_background: Color::from_rgb(0.1, 0.1, 0.1),
            playlist_background: Color::from_rgb(0.15, 0.15, 0.15),
            library_background: Color::from_rgb(0.3, 0.3, 0.3),  // Lighter color to make hot reloading visible
        }
    }
}

pub fn container_style(color: Color) -> theme::Container {
    struct CustomContainerStyle(Color);
    
    impl container::StyleSheet for CustomContainerStyle {
        type Style = Theme;
        
        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            container::Appearance {
                background: Some(Background::Color(self.0)),
                text_color: None,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }
        }
    }
    
    theme::Container::Custom(Box::new(CustomContainerStyle(color)))
}

pub fn small_text(content: &str) -> text::Text<'static> {
    text::Text::new(content.to_owned())
        .size(12)
}