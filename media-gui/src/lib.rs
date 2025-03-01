use iced::widget::{Column, Container, Row, Text, Button, progress_bar, Scrollable};
use iced::{Alignment, Element, Length, Application, Command, Theme, Subscription};
use media_audio::player::Player;
use log::info;

pub struct MediaPlayer {
    file_path: String,
    status: String,
    player: Option<Player>,
    progress: f32,
    playlist: Vec<String>,
    current_track: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FilePathChanged(String),
    PlayPressed,
    PausePressed,
    ResumePressed,
    StopPressed,
}

impl Application for MediaPlayer {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                file_path: String::from("sample.mp3"),
                status: String::from("Ready"),
                player: None,
                progress: 0.0,
                playlist: Vec::new(),
                current_track: None,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Rust Media Player")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FilePathChanged(new_path) => {
                self.file_path = new_path;
            }
            Message::PlayPressed => {
                info!("Play pressed with file: {}", self.file_path);
                if let Some(ref mut p) = self.player {
                    p.stop();
                }
                let mut new_player = Player::new();
                match new_player.play(&self.file_path) {
                    Ok(_) => {
                        self.status = format!("Playing: {}", self.file_path);
                        self.player = Some(new_player);
                        self.progress = 0.0;
                        if !self.playlist.contains(&self.file_path) {
                            self.playlist.push(self.file_path.clone());
                        }
                        self.current_track = self.playlist.iter().position(|p| p == &self.file_path);
                    }
                    Err(e) => {
                        self.status = format!("Error playing file: {}", e);
                    }
                }
            }
            Message::PausePressed => {
                if let Some(ref mut p) = self.player {
                    p.pause();
                    self.status = String::from("Paused");
                }
            }
            Message::ResumePressed => {
                if let Some(ref mut p) = self.player {
                    p.resume();
                    self.status = String::from("Resumed");
                }
            }
            Message::StopPressed => {
                if let Some(ref mut p) = self.player {
                    p.stop();
                    self.status = String::from("Stopped");
                    self.player = None;
                    self.progress = 0.0;
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // Build progress bar
        let progress_bar = progress_bar(0.0..=1.0, self.progress)
            .width(Length::Fill)
            .height(Length::Fixed(20.0));

        // Control buttons
        let play_button = Button::new(Text::new("Play"))
            .on_press(Message::PlayPressed)
            .padding(10);
        
        let pause_button = Button::new(Text::new("Pause"))
            .on_press(Message::PausePressed)
            .padding(10);
        
        let resume_button = Button::new(Text::new("Resume"))
            .on_press(Message::ResumePressed)
            .padding(10);
        
        let stop_button = Button::new(Text::new("Stop"))
            .on_press(Message::StopPressed)
            .padding(10);

        let controls = Row::with_children(vec![
            play_button.into(),
            pause_button.into(),
            resume_button.into(),
            stop_button.into(),
        ])
        .spacing(10);

        let drag_and_drop_area = Container::new(Text::new("Drag and Drop Files Here"))
            .width(Length::Fill)
            .height(Length::Fixed(50.0))
            .center_x()
            .center_y()
            .padding(10);

        let mut playlist_items = Vec::new();
        for (index, track) in self.playlist.iter().enumerate() {
            let track_name = track.split('/').last().unwrap_or(track);
            let style = if Some(index) == self.current_track {
                Container::new(Text::new(track_name))
                    .padding(5)
                    .style(iced::theme::Container::Custom(Box::new(HighlightedTrack)))
                    .width(Length::Fill)
            } else {
                Container::new(Text::new(track_name))
                    .padding(5)
                    .width(Length::Fill)
            };
            
            playlist_items.push(style.into());
        }

        let playlist_view = Scrollable::new(Column::with_children(playlist_items))
            .height(Length::Fixed(150.0))
            .width(Length::Fill);

        // Build content with reordered widgets:
        // 1. Progress bar at the top.
        // 2. Control buttons beneath the progress bar.
        // 3. Drag & drop area.
        // 4. Status text.
        // 5. Playlist view.
        let content = Column::with_children(vec![
            progress_bar.into(),
            controls.into(),
            drag_and_drop_area.into(),
            Text::new(&self.status).into(),
            playlist_view.into(),
        ])
        .padding(20)
        .align_items(Alignment::Center)
        .spacing(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

struct HighlightedTrack;

impl iced::widget::container::StyleSheet for HighlightedTrack {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: Some(iced::Color::WHITE),
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.0, 0.5, 1.0))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
        }
    }
}
