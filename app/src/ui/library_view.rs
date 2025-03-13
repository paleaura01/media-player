use iced::widget::{column, container, text, row, button, text_input, scrollable, Space};
use iced::{Element, Length, Alignment, Theme};
use core::library::LibraryState;
use crate::ui::theme::{GREEN_COLOR, DARK_GREEN_COLOR};

#[derive(Debug, Clone)]
pub enum LibraryMessage {
    None,
    AddMusicFolder,
}

// Create the library view with search functionality
pub fn view_with_search(library: &LibraryState) -> Element<LibraryMessage> {
    // Search bar at top
    let search_bar = row![
        // Search input
        text_input("Search library...", "")
            .padding(8)
            .width(Length::Fill),
            
        // View toggle buttons
        button(text("📊").size(16))
            .padding(8)
            .style(|_theme, _| button::Style {
                text_color: GREEN_COLOR,
                ..Default::default()
            }),
            
        button(text("📋").size(16))
            .padding(8)
            .style(|_theme, _| button::Style {
                text_color: GREEN_COLOR,
                ..Default::default()
            })
    ]
    .spacing(5)
    .align_y(Alignment::Center);
    
    // Album grid
    let album_grid = if library.tracks.is_empty() {
        // Empty state with add button
        let empty_content = column![
            text("No music found in library").size(16).style(|_: &Theme| text::Style {
                color: Some(GREEN_COLOR),
                ..Default::default()
            }),
            Space::with_height(20),
            button(text("+ Add Music Folder"))
                .padding(10)
                .on_press(LibraryMessage::AddMusicFolder)
                .style(|_theme, _| button::Style {
                    text_color: GREEN_COLOR,
                    border: iced::Border {
                        color: DARK_GREEN_COLOR,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
        ]
        .spacing(10)
        .align_x(Alignment::Center);
        
        // Fix: Using container without chaining into() at this level
        container(empty_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    } else {
        // Grid of albums
        let grid = scrollable(
            column(
                // Group tracks by album and create album cards
                library.tracks.iter()
                    .fold(Vec::<(&str, Vec<&core::Track>)>::new(), |mut acc, track| {
                        let album = track.album.as_deref().unwrap_or("Unknown");
                        if let Some((_, tracks)) = acc.iter_mut().find(|(a, _)| *a == album) {
                            tracks.push(track);
                        } else {
                            acc.push((album, vec![track]));
                        }
                        acc
                    })
                    .chunks(3)
                    .map(|chunk| {
                        row(
                            chunk.iter().map(|(album, tracks)| {
                                create_album_card(album, tracks)
                            }).collect::<Vec<Element<'_, LibraryMessage>>>()
                        )
                        .spacing(20)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect::<Vec<Element<'_, LibraryMessage>>>()
            )
            .spacing(20)
            .padding(20)
        );
        
        // Fix: Using container without chaining into() at this level
        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
    };
    
    // Library title and main content
    column![
        text("Library").size(20).style(|_: &Theme| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        }),
        search_bar,
        Space::with_height(10),
        album_grid
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// Helper function to create an album card - fixed lifetime issues
fn create_album_card<'a>(album: &'a str, tracks: &[&'a core::Track]) -> Element<'a, LibraryMessage> {
    let album_art = container(
        Space::new(Length::Fixed(120.0), Length::Fixed(120.0))
    )
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
        ..Default::default()
    });
    
    let album_info = column![
        text(album).size(14).style(|_: &Theme| text::Style {
            color: Some(GREEN_COLOR),
            ..Default::default()
        }),
        text(format!("{} tracks", tracks.len())).size(12).style(|_: &Theme| text::Style {
            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            ..Default::default()
        }),
    ]
    .spacing(4)
    .width(Length::Fill);
    
    column![
        album_art,
        album_info
    ]
    .spacing(5)
    .width(Length::Fixed(120.0))
    .into()
}