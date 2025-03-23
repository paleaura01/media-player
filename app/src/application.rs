// app/src/application.rs
// This is the main application file that handles events and coordinates the UI

use iced::{Element, Subscription, Task, Point};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::key::Named;
use std::path::PathBuf;
use std::fs;

// Import message types from UI modules
use crate::ui::playlist_view::PlaylistAction;
use crate::ui::library_view::LibraryMessage;

#[derive(Debug, Clone)]
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
    FileHovered,          // Fired when a file is being dragged over the window
    FileDropped(PathBuf), // Fired when a file is dropped onto the window
    FilesHoveredLeft,     // Fired when dragged files exit the window without being dropped
}

// Updated to return Task<Message> instead of void
fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    // Save shuffle state before updating
    let shuffle_was_enabled = state.player_state.shuffle_enabled;
    
    // Update player progress on every event to keep the UI current
    state.player.update_progress();
    state.player_state = state.player.get_state();
    
    // Restore shuffle state after update
    state.player_state.shuffle_enabled = shuffle_was_enabled;
    
    match message {
        Message::Action(action) => {
            // Save shuffle state before action
            let shuffle_before = state.player_state.shuffle_enabled;
            
            match action {
                core::Action::Player(player_action) => {
                    match player_action {
                        core::PlayerAction::Seek(pos) => {
                            // Handle seeking with better logging for debugging
                            println!("Directly handling seek action to position: {:.4}", pos);
                            // Call seek method directly on the player for immediate response
                            state.player.seek(pos);
                            // Update the UI state to reflect the change with exact position
                            state.player_state.progress = pos;
                            println!("Player position updated to: {:.4}", pos);
                        },
                        core::PlayerAction::SetVolume(vol) => {
                            // Handle volume changes with improved feedback
                            println!("Setting volume to: {:.2}", vol);
                            state.player.set_volume(vol);
                            
                            // Immediately update the UI state to reflect the change
                            state.player_state.volume = vol;
                            
                            println!("Volume changed to {:.2}, player and UI state updated", vol);
                        },
                        core::PlayerAction::Shuffle => {
                            // Toggle shuffle mode
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                            println!("Shuffle toggled to: {}", state.player_state.shuffle_enabled);
                        },
                        _ => {
                            // Handle other player actions
                            state.handle_action(core::Action::Player(player_action));
                            // Ensure shuffle state is preserved after action
                            state.player_state.shuffle_enabled = shuffle_before;
                        }
                    }
                },
                _ => {
                    state.handle_action(action);
                    // Ensure shuffle state is preserved after any action
                    state.player_state.shuffle_enabled = shuffle_before;
                }
            }
            
            Task::none()
        },
        Message::Playlist(action) => {
            // Save shuffle state before action
            let shuffle_before = state.player_state.shuffle_enabled;
            
            // CRITICAL FIX: Handle Seek action directly here rather than letting it get mapped to other controls
            // This avoids having the seek action get confused with next/previous track actions
            match action {
                PlaylistAction::Seek(pos) => {
                    // Direct seek handling with dedicated code path for progress bar clicks
                    println!("DIRECT SEEK from progress bar: {:.4}", pos);
                    // Call seek method directly on the player
                    state.player.seek(pos);
                    // Force UI update with the exact position
                    state.player_state.progress = pos;
                    println!("Progress bar seek: position updated to {:.4}", pos);
                    // Preserve shuffle state
                    state.player_state.shuffle_enabled = shuffle_before;
                },
                PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                    // Directly handle the PlayTrack action here
                    state.handle_action(core::Action::Playlist(
                        core::PlaylistAction::PlayTrack(playlist_id, track_idx)
                    ));
                    // Update the player state immediately BUT preserve shuffle
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                    println!("After PlayTrack, shuffle is: {}", state.player_state.shuffle_enabled);
                },
                PlaylistAction::PlayerControl(player_action) => {
                    // Handle player control actions, but skip seek handling (handled above)
                    match player_action {
                        core::PlayerAction::SetVolume(vol) => {
                            println!("Volume control from playlist action: {:.2}", vol);
                            state.player.set_volume(vol);
                            
                            // Immediately update UI state but preserve shuffle
                            state.player_state = state.player.get_state();
                            state.player_state.shuffle_enabled = shuffle_before;
                            println!("Updated volume in player state: {:.2}", state.player_state.volume);
                        },
                        // Special handling for shuffle to ensure state is preserved
                        core::PlayerAction::Shuffle => {
                            // Toggle shuffle mode directly
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                            println!("Shuffle toggled to: {}", state.player_state.shuffle_enabled);
                        },
                        // For Next/Previous track, we need special handling
                        core::PlayerAction::NextTrack | core::PlayerAction::PreviousTrack => {
                            // Handle the action
                            state.handle_action(core::Action::Player(player_action));
                            // Ensure shuffle is preserved after track change
                            state.player_state = state.player.get_state();
                            state.player_state.shuffle_enabled = shuffle_before;
                            println!("After Next/Prev, shuffle is: {}", state.player_state.shuffle_enabled);
                        },
                        _ => {
                            // Handle other player control actions
                            state.handle_action(core::Action::Player(player_action));
                            // Preserve shuffle state
                            state.player_state.shuffle_enabled = shuffle_before;
                        }
                    }
                },
                _ => {
                    // Handle other playlist actions
                    let core_action = state.playlist_view_state.handle_action(action);
                    state.handle_action(core_action);
                    // Preserve shuffle state
                    state.player_state.shuffle_enabled = shuffle_before;
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
                    
                    // Convert the path to a string (canonicalize for absolute)
                    match fs::canonicalize(&path) {
                        Ok(abs_path) => {
                            let path_str = abs_path.to_string_lossy().to_string();
                            
                            // Extract the filename to use as the track title
                            let filename = abs_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            // Check if this is an audio file by extension
                            let extension = abs_path.extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            
                            // List of supported audio formats
                            if ["mp3", "wav", "flac", "ogg", "m4a", "aac"].contains(&extension.as_str()) {
                                // Create a track with the file information
                                let track = core::Track {
                                    path: path_str,          // Store the absolute path
                                    title: Some(filename.clone()),
                                    artist: None,
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
                        },
                        Err(err) => {
                            println!("Could not resolve absolute path for dropped file: {}", err);
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