use iced::{Element, widget::{Button, Text, Row, Column, progress_bar}};
use iced::{Length, Alignment, theme};
use core::{PlayerState, PlayerAction, Action, PlaybackStatus};
use crate::ui::styles::AppStyle;

pub fn view<'a>(state: &'a PlayerState, style: &AppStyle) -> Element<'a, Action> {
    let track_info = if let Some(track) = &state.current_track {
        let filename = std::path::Path::new(track)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(track)
            .to_owned();
        Text::new(filename)
            .style(theme::Text::Color(style.colors.text_primary))
    } else {
        Text::new("No track playing")
            .style(theme::Text::Color(style.colors.text_secondary))
    };
    
    // Format position and duration as mm:ss
    let position_text = if let Some(position) = state.position {
        format!("{:02}:{:02}", position.as_secs() / 60, position.as_secs() % 60)
    } else {
        "00:00".to_string()
    };
    
    let duration_text = if let Some(duration) = state.duration {
        format!("{:02}:{:02}", duration.as_secs() / 60, duration.as_secs() % 60)
    } else {
        "00:00".to_string()
    };
    
    let progress = progress_bar(0.0..=1.0, state.progress)
        .width(Length::Fill)
        .height(Length::Fixed(20.0));
    
    let play_button = match state.status {
        PlaybackStatus::Playing => Button::new(Text::new("Pause").style(theme::Text::Color(style.colors.button_text)))
            .on_press(Action::Player(PlayerAction::Pause))
            .style(crate::ui::styles::button_style(style)),
        PlaybackStatus::Paused => Button::new(Text::new("Resume").style(theme::Text::Color(style.colors.button_text)))
            .on_press(Action::Player(PlayerAction::Resume))
            .style(crate::ui::styles::button_style(style)),
        PlaybackStatus::Stopped => {
            if let Some(track) = &state.current_track {
                Button::new(Text::new("Play").style(theme::Text::Color(style.colors.button_text)))
                    .on_press(Action::Player(PlayerAction::Play(track.clone())))
                    .style(crate::ui::styles::button_style(style))
            } else {
                Button::new(Text::new("Play").style(theme::Text::Color(style.colors.button_text)))
                    .style(crate::ui::styles::button_style(style))
            }
        },
    };
    
    let stop_button = Button::new(Text::new("Stop").style(theme::Text::Color(style.colors.button_text)))
        .on_press(Action::Player(PlayerAction::Stop))
        .style(crate::ui::styles::button_style(style));
    
    let time_display = Row::new()
        .push(Text::new(position_text).style(theme::Text::Color(style.colors.text_secondary)))
        .push(Text::new(" / ").style(theme::Text::Color(style.colors.text_secondary)))
        .push(Text::new(duration_text).style(theme::Text::Color(style.colors.text_secondary)))
        .spacing(5);
    
    let controls = Row::new()
        .push(play_button)
        .push(stop_button)
        .spacing(10)
        .align_items(Alignment::Center);
    
    Column::new()
        .push(track_info)
        .push(progress)
        .push(time_display)
        .push(controls)
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .into()
}