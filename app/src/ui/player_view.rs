// app/src/ui/player_view.rs
use iced::widget::{column, row, container, Space, slider, button};
use iced::widget::svg; // Import svg module
use iced::{Element, Length, Alignment, Theme, Border};
use core::player::PlayerState;
use crate::ui::theme::{green_text, GREEN_COLOR, DARK_GREEN_COLOR}; // Import theme helpers

#[derive(Debug, Clone)]
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
    Shuffle,
    UpdateProgress(f32),
}

// Load SVG icons as svg widgets
fn load_icon(name: &str) -> svg::Svg<iced::Theme> {
    let base_path = std::env::current_dir().unwrap_or_default();
    let icon_path = base_path.join("app").join("assets").join("icons").join(name);
    println!("Loading icon from: {}", icon_path.display());

    svg::Svg::new(svg::Handle::from_path(icon_path))
}

pub fn view(player: &PlayerState) -> Element<PlayerAction> {
    // Left section: Album art and track info
    let track_info = if let Some(track_path) = &player.current_track {
        // Extract just the filename from the path, not the entire path
        let filename = std::path::Path::new(track_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown");
        
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
                green_text(format!("Currently Playing: {}", filename)).size(16),
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
    
    // The key improvement: A slider that sends both UpdateProgress during dragging
    // AND Seek when you click (same position for both messages)
    let progress_slider = slider(0.0..=1.0, player.progress, |pos| {
        // This closure is called while dragging with the current position
        PlayerAction::UpdateProgress(pos)
    })
    .step(0.001)
    .width(Length::Fill)
    .height(15)
    // This is a hack: when released, we want to send a Seek with the SAME position
    // that was last sent in UpdateProgress
    .on_release(PlayerAction::Seek(player.progress))
    .style(|_theme: &Theme, _| slider::Style {
        rail: slider::Rail {
            backgrounds: (
                iced::Background::Color(GREEN_COLOR),
                iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.1))
            ),
            width: 15.0,
            border: Border {
                color: DARK_GREEN_COLOR,
                width: 1.0,
                radius: 3.0.into(),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: iced::Background::Color(GREEN_COLOR),
            border_width: 1.0,
            border_color: GREEN_COLOR,
        },
    });
    
    let progress_container = container(progress_slider)
        .width(Length::Fill)
        .padding([0, 5]);
    
    let progress = row![
        green_text(current_time).size(12),
        
        progress_container,
            
        green_text(total_time).size(12)
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    
    // Playback controls with volume at the right end
    let controls = row![
        // Previous track button
        button(
            load_icon("ph--skip-back-fill.svg")
                .width(20)
                .height(20)
        )
        .padding(5)
        .on_press(PlayerAction::Previous)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Rewind button
        button(
            load_icon("ph--rewind-fill.svg")
                .width(20)
                .height(20)
        )
        .padding(5)
        .on_press(PlayerAction::SkipBackward)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Play/Pause button
        if player.status == core::player::PlaybackStatus::Playing {
            button(
                load_icon("ph--pause-circle-bold.svg")
                    .width(30)
                    .height(30)
            )
            .padding(5)
            .on_press(PlayerAction::Pause)
            .style(|_theme, _| button::Style {
                background: None,
                ..Default::default()
            })
        } else {
            button(
                load_icon("ph--play-circle-bold.svg")
                    .width(30)
                    .height(30)
            )
            .padding(5)
            .on_press(PlayerAction::Play)
            .style(|_theme, _| button::Style {
                background: None,
                ..Default::default()
            })
        },
        
        // Fast-forward button
        button(
            load_icon("ph--fast-forward-fill.svg")
                .width(20)
                .height(20)
        )
        .padding(5)
        .on_press(PlayerAction::SkipForward)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Next track button
        button(
            load_icon("ph--skip-forward-fill.svg")
                .width(20)
                .height(20)
        )
        .padding(5)
        .on_press(PlayerAction::Next)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        // Shuffle button
        button(
            load_icon(if player.shuffle_enabled {
                "ph--shuffle-bold.svg"
            } else {
                "ph--shuffle-off-bold.svg"
            })
                .width(20)
                .height(20)
        )
        .padding(5)
        .on_press(PlayerAction::Shuffle)
        .style(|_theme, _| button::Style {
            background: None,
            ..Default::default()
        }),
        
        Space::with_width(20),
        
        // Volume control
        row![
            load_icon("ph--speaker-high-fill.svg")
                .width(16)
                .height(16),
            
            slider(0.0..=1.0, player.volume, PlayerAction::VolumeChange)
                .width(Length::Fixed(100.0))
                .height(6)
                .step(0.05)
                .style(|_theme: &Theme, _| slider::Style {
                    rail: slider::Rail {
                        backgrounds: (
                            iced::Background::Color(GREEN_COLOR),
                            iced::Background::Color(iced::Color::from_rgb(0.1, 0.1, 0.1))
                        ),
                        width: 6.0,
                        border: Border {
                            color: DARK_GREEN_COLOR,
                            width: 1.0,
                            radius: 3.0.into(),
                        },
                    },
                    handle: slider::Handle {
                        shape: slider::HandleShape::Circle { radius: 7.0 },
                        background: iced::Background::Color(GREEN_COLOR),
                        border_width: 1.0,
                        border_color: GREEN_COLOR,
                    },
                })
        ]
        .spacing(5)
        .align_y(Alignment::Center)
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    
    // Overall layout
    let content = column![
        row![
            track_info.width(Length::FillPortion(3)),
            Space::with_width(25),
            progress.width(Length::FillPortion(4)),
            Space::with_width(25),
            controls.width(Length::FillPortion(5))
        ]
        .padding(10)
        .spacing(20)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    ]
    .spacing(10)
    .width(Length::Fill);
    
    content.into()
}