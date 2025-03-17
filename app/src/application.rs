// app/src/application.rs
// This is the main application file that handles events and coordinates the UI

use iced::{Element, Subscription, Task, Point};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::key::Named;
use std::path::PathBuf;

// Import message types from UI modules
use crate::ui::playlist_view::PlaylistAction;
use crate::ui::library_view::LibraryMessage;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    /// Core action messages
    Action(core::Action),
    /// Playlist messages from the UI
    Playlist(PlaylistAction),
    /// Library messages
    Library(LibraryMessage),
    /// Folder selection result
    FolderSelected(Option<PathBuf>),
    /// Window events
    WindowClosed { x: i32, y: i32 },
    /// Mouse position for hover detection
    MousePosition(Point),
    /// Window focus events - important for keeping selection when window isn't active
    WindowFocusLost,
    WindowFocusGained,
    /// File drag and drop events - these enable adding tracks via drag-and-drop
    FileHovered,             // Fired when a file is being dragged over the window
    FileDropped(PathBuf),    // Fired when a file is dropped onto the window
    FilesHoveredLeft,        // Fired when dragged files exit the window without being dropped
}


// Updated to return Task<Message> instead of void
fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    // Update player progress on every event to keep the UI current
    state.player.update_progress();
    state.player_state = state.player.get_state();
    
    match message {
        Message::Action(action) => {
            match action {
                core::Action::Player(player_action) => {
                    match player_action {
                        core::PlayerAction::Seek(pos) => {
                            // Handle seeking
                            state.player.seek(pos);
                        },
                        core::PlayerAction::SetVolume(vol) => {
                            // Handle volume changes
                            state.player.set_volume(vol);
                        },
                        _ => state.handle_action(core::Action::Player(player_action)),
                    }
                },
                _ => state.handle_action(action),
            }
            Task::none()
        },
        Message::Playlist(action) => {
            // Direct handling of PlayTrack to bypass potential issues
            match action {
                PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                    // Directly handle the PlayTrack action here
                    state.handle_action(core::Action::Playlist(
                        core::PlaylistAction::PlayTrack(playlist_id, track_idx)
                    ));
                    // Update the player state immediately
                    state.player_state = state.player.get_state();
                }
                PlaylistAction::PlayerControl(player_action) => {
                    // Handle player control actions from the UI
                    state.handle_action(core::Action::Player(player_action));
                    // Update the player state immediately
                    state.player_state = state.player.get_state();
                }
                _ => {
                    // Handle other playlist actions as before
                    let core_action = state.playlist_view_state.handle_action(action);
                    state.handle_action(core_action);
                }
            }
            Task::none()
        },
        Message::Library(LibraryMessage::AddMusicFolder) => {
            // Ideally you'd use rfd here, but for now we'll manually implement
            // by using a core library action
            state.handle_action(core::Action::Library(
                core::LibraryAction::StartScan
            ));
            Task::none()
        },
        Message::Library(LibraryMessage::None) => {
            Task::none()
        },
        Message::Library(LibraryMessage::ToggleView) => {
            // Handle the toggle view action here
            // For now, we'll just do nothing as the view toggle functionality
            // isn't fully implemented
            Task::none()
        },
        Message::FolderSelected(Some(path)) => {
            if let Some(path_str) = path.to_str() {
                state.handle_action(core::Action::Library(
                    core::LibraryAction::AddScanDirectory(path_str.to_string())
                ));
                state.handle_action(core::Action::Library(
                    core::LibraryAction::StartScan
                ));
            }
            Task::none()
        },
        Message::FolderSelected(None) => {
            Task::none()
        },
        Message::WindowClosed { x, y } => {
            if let Err(e) = window_state::save_window_position(x, y) {
                log::error!("Failed to save window position: {}", e);
            }
            Task::none()
        },
        Message::MousePosition(_position) => {
            // We don't change selection state based on mouse position
            Task::none()
        },
        Message::WindowFocusLost => {
            // Intentionally do nothing - keep selection state
            println!("Window focus lost - maintaining selection state");
            Task::none()
        },
        Message::WindowFocusGained => {
            // Intentionally do nothing - keep selection state
            println!("Window focus gained - maintaining selection state");
            Task::none()
        },
        Message::FileHovered => {
            // Visual feedback could be added here (like highlighting the drop zone)
            println!("File is being hovered over the window");
            Task::none()
        },
        Message::FileDropped(path) => {
            println!("File dropped: {:?}", path);
            
            // Check if there's a selected playlist to add the track to
            if let Some(selected_idx) = state.playlists.selected {
                if selected_idx < state.playlists.playlists.len() {
                    let playlist_id = state.playlists.playlists[selected_idx].id;
                    
                    // Convert the path to a string
                    if let Some(path_str) = path.to_str() {
                        // Extract the filename to use as the track title
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        // Check if this is an audio file by examining the extension
                        let extension = path.extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        
                        // List of supported audio formats
                        if ["mp3", "wav", "flac", "ogg", "m4a", "aac"].contains(&extension.as_str()) {
                            // Create a track with the file information
                            // The path needs to be stored as a string for later playback
                            let track = core::Track {
                                path: path_str.to_string(),  // Store the full path for playback
                                title: Some(filename.clone()),  // Clone filename so we can use it again below
                                artist: None,                // These could be populated by metadata later
                                album: None,
                            };
                            
                            // Add the track to the selected playlist
                            state.handle_action(core::Action::Playlist(
                                core::PlaylistAction::AddTrack(playlist_id, track)
                            ));
                            
                            // Log success
                            println!("Added track to playlist {}: {}", playlist_id, filename);
                        } else {
                            println!("Dropped file is not a supported audio format: {}", extension);
                        }
                    }
                }
            } else {
                println!("No playlist selected to add the track to");
            }
            
            Task::none()
        },
        Message::FilesHoveredLeft => {
            // Clean up any visual feedback 
            println!("Files no longer being hovered over the window");
            Task::none()
        },
    }
}

fn view(state: &MediaPlayer) -> Element<Message> {
    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        &state.playlist_view_state,
    );

    rendered.map(Message::Playlist)
}

// Just listen for events - we'll update the player on each event
fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    // Event listener for keyboard, mouse, etc.
    iced::event::listen().map(|event| {
        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                match key {
                    iced::keyboard::Key::Named(Named::Space) => {
                        Message::Action(core::Action::Player(core::PlayerAction::Pause))
                    },
                    iced::keyboard::Key::Named(Named::Escape) => {
                        Message::Action(core::Action::Player(core::PlayerAction::Stop))
                    },
                    _ => Message::Playlist(PlaylistAction::None)
                }
            },
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Message::MousePosition(position)
            },
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                Message::WindowClosed { x: 100, y: 100 }
            },
            iced::Event::Window(iced::window::Event::Moved(position)) => {
                let x = position.x as i32;
                let y = position.y as i32;
                Message::WindowClosed { x, y }
            },
            iced::Event::Window(iced::window::Event::Unfocused) => {
                Message::WindowFocusLost
            },
            iced::Event::Window(iced::window::Event::Focused) => {
                Message::WindowFocusGained
            },
            // Handle file drag and drop events
            iced::Event::Window(iced::window::Event::FileHovered(_)) => {
                Message::FileHovered
            },
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Message::FileDropped(path)
            },
            iced::Event::Window(iced::window::Event::FilesHoveredLeft) => {
                Message::FilesHoveredLeft
            },
            _ => {
                // For any other events, send a None action that doesn't change application state
                Message::Playlist(PlaylistAction::None)
            }
        }
    })
}

pub fn run() -> iced::Result {
    iced::application("Media Player", update, view)
        .subscription(subscription)
        .window(window_state::window_settings())
        .theme(|_state| ui::theme::dark_theme())
        .run()
}