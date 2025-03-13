use iced::widget::column;
use iced::Element;
use core::player::PlayerState;
use crate::ui::theme::{green_text, green_button, green_progress_bar};

#[derive(Debug, Clone)]
pub enum PlayerAction {
    Play,
    Pause, // Add the missing Pause variant
    Stop,
    None,
}

pub fn view(player: &PlayerState) -> Element<PlayerAction> {
    let label = if let Some(track) = &player.current_track {
        green_text(format!("Currently Playing: {}", track))
    } else {
        green_text("No track playing")
    };

    let progress = green_progress_bar(0.0, 100.0);

    let controls = column![
        green_button("Play", PlayerAction::Play),
        green_button("Pause", PlayerAction::Pause), // Add Pause button
        green_button("Stop", PlayerAction::Stop),
    ]
    .spacing(10);

    column![
        label,
        progress,
        controls,
    ]
    .spacing(10)
    .into()
}