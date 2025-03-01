// src/gui.rs
use iced::{
    widget::{button, column, container, row, text, text_input},
    Application, Command, Element, Length, Theme,
};

use crate::player::{Player, PlayerState};

#[derive(Debug, Clone)]
pub enum Message {
    Play,
    Pause,
    Stop,
    InputChanged(String),
}

pub struct MediaPlayer {
    player: Player,
    file_path: String,
}

impl Application for MediaPlayer {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                player: Player::new(),
                file_path: String::from("sample.mp3"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Media Player")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Play => {
                if self.player.get_state() == PlayerState::Paused {
                    self.player.resume();
                } else if self.player.get_state() == PlayerState::Stopped {
                    let _ = self.player.play(&self.file_path);
                }
            }
            Message::Pause => {
                self.player.pause();
            }
            Message::Stop => {
                self.player.stop();
            }
            Message::InputChanged(value) => {
                self.file_path = value;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let input = text_input("Enter file path...", &self.file_path)
            .padding(10)
            .on_input(Message::InputChanged);

        let play_button = button("Play")
            .padding(10)
            .on_press(Message::Play);

        let pause_button = button("Pause")
            .padding(10)
            .on_press(Message::Pause);

        let stop_button = button("Stop")
            .padding(10)
            .on_press(Message::Stop);

        let controls = row![play_button, pause_button, stop_button]
            .spacing(10);

        container(
            column![
                text("Media Player").size(30),
                input,
                controls
            ]
            .padding(20)
            .spacing(20)
            .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}