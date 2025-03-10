// app/src/ui/player_view.rs
use iced::{Element, widget::{Button, Text, Row, Column, progress_bar}, Length, Alignment};
use core::{PlayerState, PlayerAction, Action, PlaybackStatus};
use crate::ui::styles::AppStyle;

pub fn view(state: &PlayerState, _style: &AppStyle) -> Element<'static, Action> {
    let track_info = if let Some(track) = &state.current_track {
        let filename = std::path::Path::new(track)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(track)
            .to_owned();
        Text::new(filename)
    } else {
        Text::new("No track playing")
    };
    let progress = progress_bar(0.0..=1.0, state.progress)
        .width(Length::Fill)
        .height(Length::Fixed(20.0));
    let play_button = match state.status {
        PlaybackStatus::Playing => Button::new(Text::new("Pause"))
            .on_press(Action::Player(PlayerAction::Pause)),
        PlaybackStatus::Paused => Button::new(Text::new("Resume"))
            .on_press(Action::Player(PlayerAction::Resume)),
        PlaybackStatus::Stopped => {
            if let Some(track) = &state.current_track {
                Button::new(Text::new(track.clone()))
                    .on_press(Action::Player(PlayerAction::Play(track.clone())))
            } else {
                Button::new(Text::new("Play"))
            }
        },
    };
    let stop_button = Button::new(Text::new("Stop"))
        .on_press(Action::Player(PlayerAction::Stop));
    let controls = Row::new()
        .push(play_button)
        .push(stop_button)
        .spacing(10)
        .align_y(Alignment::Center);
    Column::new()
        .push(track_info)
        .push(progress)
        .push(controls)
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .into()
}
