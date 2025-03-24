// app/src/application.rs
// This is the main application file that handles events and coordinates the UI

use iced::{Element, Subscription, Task, Point};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::{Key, key::Named};
use std::path::PathBuf;

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
                            // DEBUG OUTPUT for direct seek action
                            println!("██ DEBUG: Direct Seek action received: {:.4}", pos);
                            
                            // Call seek method directly on the player
                            println!("██ DEBUG: Calling state.player.seek({:.4})", pos);
                            state.player.seek(pos);
                            
                            // Force immediate UI update for better responsiveness
                            state.player_state.progress = pos;
                            
                            println!("██ DEBUG: Seek completed, state.player_state.progress = {:.4}", 
                                    state.player_state.progress);
                        },
                        core::PlayerAction::SetVolume(vol) => {
                            // Handle volume changes
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                        },
                        core::PlayerAction::Shuffle => {
                            // Toggle shuffle mode
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
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
            
            // CRITICAL FIX: Directly handle Seek action here
            match action {
                PlaylistAction::Seek(pos) => {
                    // Debug output to trace the seek action flow
                    println!("██ DEBUG: PlaylistAction::Seek({:.4}) received in application.rs", pos);
                    
                    // Use direct player API call for seeking - this bypasses the normal action flow
                    println!("██ DEBUG: Directly calling state.player.seek({:.4})", pos);
                    state.player.seek(pos);
                    
                    // Update UI state immediately
                    println!("██ DEBUG: Setting state.player_state.progress = {:.4}", pos);
                    state.player_state.progress = pos;
                    
                    // Preserve shuffle state
                    state.player_state.shuffle_enabled = shuffle_before;
                    
                    println!("██ DEBUG: Seek processing complete in application.rs");
                },
                PlaylistAction::PlayTrack(playlist_id, track_idx) => {
                    // Handle the PlayTrack action directly
                    state.handle_action(core::Action::Playlist(
                        core::PlaylistAction::PlayTrack(playlist_id, track_idx)
                    ));
                    
                    // Update player state but preserve shuffle
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                },
                PlaylistAction::PlayerControl(player_action) => {
                    // Handle volume separately
                    match player_action {
                        core::PlayerAction::SetVolume(vol) => {
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                        },
                        core::PlayerAction::Shuffle => {
                            // Toggle shuffle mode directly
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                        },
                        _ => {
                            // Handle other player actions
                            state.handle_action(core::Action::Player(player_action));
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
        // Other message handlers remain the same
        Message::Library(LibraryMessage::AddMusicFolder) => {
            state.handle_action(core::Action::Library(
                core::LibraryAction::StartScan
            ));
            Task::none()
        },
        Message::Library(LibraryMessage::ToggleView) => {
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
            Task::none()
        },
        Message::WindowFocusLost => {
            Task::none()
        },
        Message::WindowFocusGained => {
            Task::none()
        },
        Message::FileHovered => {
            Task::none()
        },
        Message::FileDropped(path) => {
            println!("File dropped: {:?}", path);
            
            if let Some(selected_idx) = state.playlists.selected {
                if selected_idx < state.playlists.playlists.len() {
                    let playlist_id = state.playlists.playlists[selected_idx].id;
                    
                    // Convert to absolute path
                    match std::fs::canonicalize(&path) {
                        Ok(abs_path) => {
                            let path_str = abs_path.to_string_lossy().to_string();
                            
                            let filename = abs_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            let extension = abs_path.extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            
                            // Check for supported formats
                            if ["mp3", "wav", "flac", "ogg", "m4a", "aac"].contains(&extension.as_str()) {
                                let track = core::Track {
                                    path: path_str,
                                    title: Some(filename.clone()),
                                    artist: None,
                                    album: None,
                                };
                                // Add to selected playlist
                                state.handle_action(core::Action::Playlist(
                                    core::PlaylistAction::AddTrack(playlist_id, track)
                                ));
                            } else {
                                println!("Not a supported audio format: {}", extension);
                            }
                        },
                        Err(err) => {
                            println!("Failed to canonicalize dropped file path: {}", err);
                        }
                    }
                }
            } else {
                println!("No playlist selected to add the track to");
            }
            Task::none()
        },
        Message::FilesHoveredLeft => {
            Task::none()
        },
    }
}

// The rest remains the same
fn view(state: &MediaPlayer) -> Element<Message> {
    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        &state.playlist_view_state,
    );

    rendered.map(Message::Playlist)
}

fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    iced::event::listen().map(|event| {
        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                match key {
                    Key::Named(Named::Space) => {
                        Message::Action(core::Action::Player(core::PlayerAction::Pause))
                    },
                    Key::Named(Named::Escape) => {
                        Message::Action(core::Action::Player(core::PlayerAction::Stop))
                    },
                    _ => Message::Playlist(PlaylistAction::None),
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