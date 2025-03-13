// app/src/ui/player_view.rs
use iced::widget::{column, row, container, Space, text, slider, button, image};
use iced::{Element, Length, Alignment, Theme};
use core::player::PlayerState;
use crate::ui::theme::green_text;

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

// Function to load an icon with proper logging
fn load_icon(name: &str) -> image::Handle {
    let base_path = std::env::current_dir().unwrap_or_default();
    let icon_path = base_path.join("app").join("assets").join("icons").join(name);
    
    // Log the full path for debugging
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
    
    // Load icon images with better error handling
    let prev_icon = load_icon("ph--skip-back-thin.svg");
    let rewind_icon = load_icon("ph--rewind-thin.svg");
    let play_icon = load_icon("ph--play-circle-thin.svg");
    let pause_icon = load_icon("ph--pause-circle-thin.svg");
    let forward_icon = load_icon("ph--fast-forward-thin.svg");
    let next_icon = load_icon("ph--skip-forward-thin.svg");
    let volume_icon = load_icon("ph--speaker-simple-high-thin.svg");
    
    // Right: Playback controls and volume
    let controls = row![
        // Previous track - smaller button
        button(
            image(prev_icon)
                .width(24)
                .height(24)
        )
        .padding(5)
        .on_press(PlayerAction::Previous)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Rewind - smaller button
        button(
            image(rewind_icon)
                .width(24)
                .height(24)
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
                image(pause_icon)
                    .width(36)
                    .height(36)
            )
            .padding(5)
            .on_press(PlayerAction::Pause)
            .style(|_theme, _| button::Style {
                background: None,
                ..Default::default()
            })
        } else {
            button(
                image(play_icon)
                    .width(36)
                    .height(36)
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
            image(forward_icon)
                .width(24)
                .height(24)
        )
        .padding(5)
        .on_press(PlayerAction::SkipForward)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Next track - smaller button
        button(
            image(next_icon)
                .width(24)
                .height(24)
        )
        .padding(5)
        .on_press(PlayerAction::Next)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Volume slider with icon
        row![
            image(volume_icon)
                .width(20)
                .height(20),
            
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