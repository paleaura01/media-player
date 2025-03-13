use iced::widget::{column, row, container, Space};
use iced::{Element, Length, Alignment};
use core::player::PlayerState;
use crate::ui::theme::{green_text, green_button, green_progress_bar, player_container_style};

#[derive(Debug, Clone)]
pub enum PlayerAction {
    Play,
    Pause,
    Stop,
    #[allow(dead_code)]  // Add this attribute to suppress the warning
    None,
}

pub fn view(player: &PlayerState) -> Element<PlayerAction> {
    // Track info with better styling
    let label = if let Some(track) = &player.current_track {
        green_text(format!("Currently Playing: {}", track))
    } else {
        green_text("No track playing")
    };

    // Progress bar with proper padding and width
    let progress = container(green_progress_bar(0.0, 100.0))
        .width(Length::Fill)
        .padding(10);

    // Controls in a horizontal row with spacing
    let controls = row![
        green_button("Play", PlayerAction::Play),
        Space::with_width(10),
        green_button("Pause", PlayerAction::Pause),
        Space::with_width(10),
        green_button("Stop", PlayerAction::Stop),
    ]
    .spacing(5)
    .align_y(Alignment::Center);

    // Overall player layout
    let content = column![
        label,
        progress,
        controls,
    ]
    .spacing(15)
    .padding(20)
    .width(Length::Fill)
    .align_x(Alignment::Center);
    
    // Wrap in container with the darker background style to match playlist
    container(content)
        .width(Length::Fill)
        .style(player_container_style())
        .into()
}