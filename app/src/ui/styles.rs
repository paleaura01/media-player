use iced::{Color, Background, widget::{container, text, button}};
use iced::{Theme, theme};

// Increment this version number whenever you update UI to trigger hot-reloading
#[no_mangle]
pub static UI_VERSION_INT: u64 = 4;

// Updated version to test hot reloading
pub const UI_VERSION: &str = "4.0.0";

#[derive(Debug, Clone)]
pub struct AppStyle {
    pub colors: ColorTheme,
}

#[derive(Debug, Clone)]
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

// Winamp-inspired color scheme
pub fn default_style() -> AppStyle {
    AppStyle {
        colors: ColorTheme {
            player_background: Color::from_rgb(0.12, 0.12, 0.12),  // Dark gray
            playlist_background: Color::from_rgb(0.16, 0.16, 0.18), // Slightly lighter gray
            library_background: Color::from_rgb(0.16, 0.16, 0.18),  // Same as playlist
            text_primary: Color::from_rgb(0.9, 0.9, 0.9),          // Light gray/white
            text_secondary: Color::from_rgb(0.7, 0.7, 0.7),        // Medium gray
            button_background: Color::from_rgb(0.2, 0.2, 0.25),    // Dark blue-gray
            button_hover: Color::from_rgb(0.25, 0.25, 0.3),        // Slightly lighter
            button_text: Color::from_rgb(0.9, 0.9, 0.9),           // Light gray/white
            highlight: Color::from_rgb(0.35, 0.65, 0.85),          // Winamp blue
            border: Color::from_rgb(0.25, 0.25, 0.3),              // Border color
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
        .style(theme::Text::Color(Color::from_rgb(0.7, 0.7, 0.7))) // Fixed: use style instead of color
}

pub fn button_style(style: &AppStyle) -> theme::Button {
    struct CustomButtonStyle {
        background: Color,
        text: Color,
        border: Color,
        hover: Color,
    }
    
    impl button::StyleSheet for CustomButtonStyle {
        type Style = Theme;
        
        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(Background::Color(self.background)),
                text_color: self.text,
                border_radius: 2.0,
                border_width: 1.0,
                border_color: self.border,
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
        
        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let active = self.active(style);
            button::Appearance {
                background: Some(Background::Color(self.hover)),
                ..active
            }
        }
    }
    
    theme::Button::Custom(Box::new(CustomButtonStyle {
        background: style.colors.button_background,
        text: style.colors.button_text, 
        border: style.colors.border,
        hover: style.colors.button_hover,
    }))
}

// New style for selected playlists
pub fn selected_button_style(style: &AppStyle) -> theme::Button {
    struct SelectedButtonStyle {
        background: Color,
        text: Color,
        border: Color,
    }
    
    impl button::StyleSheet for SelectedButtonStyle {
        type Style = Theme;
        
        fn active(&self, _style: &Self::Style) -> button::Appearance {
            button::Appearance {
                background: Some(Background::Color(self.background)),
                text_color: self.text,
                border_radius: 2.0,
                border_width: 1.0,
                border_color: self.border,
                shadow_offset: iced::Vector::new(0.0, 0.0),
            }
        }
        
        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            self.active(style)
        }
    }
    
    theme::Button::Custom(Box::new(SelectedButtonStyle {
        background: style.colors.highlight,
        text: style.colors.text_primary, 
        border: style.colors.border,
    }))
}