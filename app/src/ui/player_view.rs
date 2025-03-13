// app/src/ui/player_view.rs
use iced::widget::{column, row, container, Space, text, slider, button, image};
use iced::{Element, Length, Alignment, Theme};
use core::player::PlayerState;
use crate::ui::theme::{green_text, GREEN_COLOR};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PlayerAction {
    Play,
    Pause,
    Stop,
    SkipForward,
    SkipBackward,
    Next,
    Previous,
    VolumeChange(f32),
    Seek(f32),
}

// Helper function to load SVG icons with absolute path
fn load_icon(name: &str) -> image::Handle {
    let base_path = std::env::current_dir().unwrap_or_default();
    let icon_path = base_path.join("app").join("assets").join("icons").join(name);
    
    // This will print the full path for debugging
    println!("Loading icon from: {}", icon_path.display());
    
    image::Handle::from_path(icon_path)
}

pub fn view(player: &PlayerState) -> Element<PlayerAction> {
    // Left section: Album art and track info
    let track_info = if let Some(track) = &player.current_track {
        row![
            // Album art placeholder
            container(
                Space::new(Length::Fixed(50.0), Length::Fixed(50.0))
            )
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
                ..Default::default()
            }),
            
            Space::with_width(10),
            
            column![
                green_text(format!("Currently Playing: {}", track)).size(16),
                green_text("Artist - Album").size(12),
            ]
            .spacing(4)
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    } else {
        row![
            // Empty album art placeholder
            container(
                Space::new(Length::Fixed(50.0), Length::Fixed(50.0))
            )
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
                ..Default::default()
            }),
            
            Space::with_width(10),
            
            green_text("No track playing").size(16)
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    };
    
    // Center: Progress bar with time indicators
    let current_time = player.position.map_or("0:00".to_string(), |pos| {
        let secs = pos.as_secs();
        format!("{}:{:02}", secs / 60, secs % 60)
    });
    
    let total_time = player.duration.map_or("0:00".to_string(), |dur| {
        let secs = dur.as_secs();
        format!("{}:{:02}", secs / 60, secs % 60)
    });
    
    // Create a progress bar
    let progress_bar = slider(0.0..=1.0, player.progress, PlayerAction::Seek)
        .width(Length::Fill);
    
    let progress = row![
        text(current_time).size(12).style(|_: &Theme| text::Style {
            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            ..Default::default()
        }),
        
        progress_bar,
            
        text(total_time).size(12).style(|_: &Theme| text::Style {
            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            ..Default::default()
        })
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    
    // Right: Playback controls and volume with text-based icons
    let controls = row![
        // Previous track - smaller button
        button(
            text("⏮").size(20).style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            })
        )
        .padding(5)
        .on_press(PlayerAction::Previous)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Rewind - smaller button
        button(
            text("⏪").size(20).style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            })
        )
        .padding(5)
        .on_press(PlayerAction::SkipBackward)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Play/Pause - larger button
        if player.status == core::player::PlaybackStatus::Playing {
            button(
                text("⏸").size(30).style(|_: &Theme| text::Style {
                    color: Some(GREEN_COLOR),
                    ..Default::default()
                })
            )
            .padding(5)
            .on_press(PlayerAction::Pause)
            .style(|_theme, _| button::Style {
                background: None,
                ..Default::default()
            })
        } else {
            button(
                text("▶").size(30).style(|_: &Theme| text::Style {
                    color: Some(GREEN_COLOR),
                    ..Default::default()
                })
            )
            .padding(5)
            .on_press(PlayerAction::Play)
            .style(|_theme, _| button::Style {
                background: None,
                ..Default::default()
            })
        },
        
        // Fast-forward - smaller button
        button(
            text("⏩").size(20).style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            })
        )
        .padding(5)
        .on_press(PlayerAction::SkipForward)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Next track - smaller button
        button(
            text("⏭").size(20).style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            })
        )
        .padding(5)
        .on_press(PlayerAction::Next)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Volume slider with icon
        row![
            text("🔊").size(16).style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            }),
            
            slider(0.0..=1.0, player.volume, PlayerAction::VolumeChange)
                .width(Length::Fixed(100.0))
        ]
        .spacing(5)
        .align_y(Alignment::Center)
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    
    // Overall player layout
    let content = column![
        // Main row with track info, progress, and controls
        row![
            track_info.width(Length::FillPortion(3)),
            Space::with_width(20),
            progress.width(Length::FillPortion(5)),
            Space::with_width(20),
            controls.width(Length::FillPortion(4))
        ]
        .padding(10)
        .spacing(20)
        .align_y(Alignment::Center)
        .width(Length::Fill)
    ]
    .spacing(10)
    .width(Length::Fill);
    
    content.into()
}