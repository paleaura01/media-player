use iced::{Element, widget::{Container, Button, Text, Row, Column, progress_bar}};
use iced::{Length, Alignment};
use core::{PlayerState, PlayerAction, Action, PlaybackStatus};

pub fn view(state: &PlayerState) -> Element<'static, Action> {
    // Display track info
    let track_info = if let Some(track) = &state.current_track {
        let filename = std::path::Path::new(track)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(track);
        
        Text::new(filename)
    } else {
        Text::new("No track playing")
    };
    
    // Progress bar
    let progress = progress_bar(0.0..=1.0, state.progress)
        .width(Length::Fill)
        .height(Length::Fixed(20.0));
    
    // Create buttons based on player state
    let play_button = match state.status {
        PlaybackStatus::Playing => {
            Button::new(Text::new("Pause"))
                .on_press(Action::Player(PlayerAction::Pause))
        },
        PlaybackStatus::Paused => {
            Button::new(Text::new("Resume"))
                .on_press(Action::Player(PlayerAction::Resume))
        },
        PlaybackStatus::Stopped => {
            if let Some(track) = &state.current_track {
                Button::new(Text::new("Play"))
                    .on_press(Action::Player(PlayerAction::Play(track.clone())))
            } else {
                Button::new(Text::new("Play"))
            }
        },
    };
    
    let stop_button = Button::new(Text::new("Stop"))
        .on_press(Action::Player(PlayerAction::Stop));
    
    // Controls row
    let controls = Row::new()
        .push(play_button)
        .push(stop_button)
        .spacing(10)
        .align_items(Alignment::Center);
    
    // Full player section
    Column::new()
        .push(track_info)
        .push(progress)
        .push(controls)
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .into()
}