use iced::{
    window::{self, icon},
    Point, Subscription,
};
use log::info;

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
}

#[derive(Default)]
pub struct Counter {
    value: u64,
}

// A simple update function
pub fn update(counter: &mut Counter, message: Message) {
    match message {
        Message::Increment => counter.value += 1,
    }
}

// A simple view function
use iced::{Element, widget::{button, text}};
pub fn view(counter: &Counter) -> Element<Message> {
    button(text(counter.value))
        .on_press(Message::Increment)
        .into()
}

// (Optional) If you want a subscription in the same file:
pub fn subscription(_state: &Counter) -> Subscription<Message> {
    Subscription::none()
}

// Bright green button style (optional)
use iced::widget::button::Style as ButtonStyle;
use iced::{Color, Background, Vector, Border, Shadow};
pub fn bright_green_button_style() -> ButtonStyle {
    ButtonStyle {
        background: Some(Background::Color(Color::from_rgb(0.1, 1.0, 0.1))),
        border: Border {
            width: 2.0,
            color: Color::from_rgb(0.0, 0.8, 0.0),
            radius: 6.0.into(),
        },
        text_color: Color::WHITE,
        shadow: Shadow {
            offset: Vector::new(1.0, 1.0),
            blur_radius: 2.0,
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
        },
        ..Default::default()
    }
}

/// Run the entire iced application, including a custom icon.
pub fn run_app() -> iced::Result {
    // Set up logging
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("Starting media player application (icon_state).");

    // Load icon data from the file (relative path fixed)
    let icon_bytes = include_bytes!("../../assets/icon.ico");

    // Create the icon using from_file_data (available via the "image" feature)
    let app_icon = icon::from_file_data(icon_bytes, None)
        .expect("Failed to load icon data from file bytes!");

    // Build the iced application and set the window icon.
    iced::application("Media Player (IconConfig)", update, view)
        .window(window::Settings {
            icon: Some(app_icon),
            position: window::Position::Specific(Point::new(300.0, 200.0)),
            ..window::Settings::default()
        })
        .subscription(subscription)
        .run()
}
