use iced::{Element, Subscription, Task, Point};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::{Key, key::Named};
use std::path::PathBuf;

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
    /// Window focus events
    WindowFocusLost,
    WindowFocusGained,
    /// File drag and drop events
    FileHovered,
    FileDropped(PathBuf),
    FilesHoveredLeft,
}

fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    // Preserve shuffle between updates
    let shuffle_before = state.player_state.shuffle_enabled;

    // Always update our local copy of the player's state
    state.player.update_progress();
    state.player_state = state.player.get_state();
    state.player_state.shuffle_enabled = shuffle_before;

    match message {
        Message::Action(action) => {
            let shuffle_before = state.player_state.shuffle_enabled;
            match action {
                core::Action::Player(player_action) => {
                    match player_action {
                        // We do not do direct seeking here anymore.
                        // We'll just do request_seek in the decoder loop approach.
                        core::PlayerAction::Seek(pos) => {
                            // First clear buffers
                            state.player.clear_audio_buffers();
                            
                            // Then request the seek
                            if let Ok(mut lock) = state.player.playback_position.lock() {
                                lock.request_seek(pos);
                                log::debug!("(Action) Requested seek to {:.4} - lock acquired successfully", pos);
                            } else {
                                log::error!("(Action) Failed to acquire lock for seek to {:.4}", pos);
                            }
                            
                            // Update UI state immediately for responsiveness
                            state.player_state.progress = pos;
                            log::debug!("(Action) Updated UI progress to {:.4}", pos);
                        },
                        core::PlayerAction::SetVolume(vol) => {
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                        },
                        core::PlayerAction::Shuffle => {
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                        },
                        _ => {
                            state.handle_action(core::Action::Player(player_action));
                            state.player_state.shuffle_enabled = shuffle_before;
                        }
                    }
                },
                _ => {
                    state.handle_action(action);
                    state.player_state.shuffle_enabled = shuffle_before;
                }
            }
            Task::none()
        },
        Message::Playlist(action) => {
            match action {
                PlaylistAction::Seek(pos) => {
                    // More detailed logging
                    log::info!("(Playlist) Received Seek({:.4}) -> Requesting seek in playback_position", pos);
                    
                    // Clear any buffers first
                    state.player.clear_audio_buffers();
                    
                    // Request the seek with proper locking
                    if let Ok(mut lock) = state.player.playback_position.lock() {
                        lock.request_seek(pos);
                        log::debug!("Seek request successfully set for position {:.4}", pos);
                    } else {
                        log::error!("Failed to acquire lock for seek request");
                    }
                    
                    // Update the UI state immediately for responsiveness
                    state.player_state.progress = pos;
                    log::debug!("Updated UI progress to {:.4}", pos);
                },
                PlaylistAction::UpdateProgress(pos) => {
                    // Just update the UI progress without initiating a seek
                    state.player_state.progress = pos;
                    log::debug!("Updated UI progress during dragging to {:.4}", pos);
                },
                PlaylistAction::PlayTrack(pid, tid) => {
                    log::info!("Playing track {} from playlist {}", tid, pid);
                    state.handle_action(core::Action::Playlist(core::PlaylistAction::PlayTrack(pid, tid)));
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                },
                PlaylistAction::PlayerControl(ctrl) => {
                    match ctrl {
                        core::PlayerAction::SetVolume(vol) => {
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                        },
                        core::PlayerAction::Shuffle => {
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                        },
                        core::PlayerAction::Seek(pos) => {
                            // Clear buffers first
                            state.player.clear_audio_buffers();
                            
                            // Then request the seek
                            if let Ok(mut lock) = state.player.playback_position.lock() {
                                lock.request_seek(pos);
                                log::debug!("(PlayerControl) Seek request set for position {:.4}", pos);
                            } else {
                                log::error!("(PlayerControl) Failed to acquire lock for seek request");
                            }
                            
                            // Update UI immediately
                            state.player_state.progress = pos;
                        },
                        _ => {
                            state.handle_action(core::Action::Player(ctrl));
                            state.player_state = state.player.get_state();
                            state.player_state.shuffle_enabled = shuffle_before;
                        }
                    }
                },
                _ => {
                    // other playlist actions
                    let core_action = state.playlist_view_state.handle_action(action);
                    state.handle_action(core_action);
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                }
            }
            Task::none()
        },
        Message::Library(LibraryMessage::AddMusicFolder) => {
            state.handle_action(core::Action::Library(core::LibraryAction::StartScan));
            Task::none()
        },
        Message::Library(LibraryMessage::ToggleView) => {
            Task::none()
        },
        Message::FolderSelected(Some(path)) => {
            if let Some(path_str) = path.to_str() {
                state.handle_action(core::Action::Library(
                    core::LibraryAction::AddScanDirectory(path_str.to_string()),
                ));
                state.handle_action(core::Action::Library(core::LibraryAction::StartScan));
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
        Message::MousePosition(_pos) => {
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
            log::info!("File dropped: {:?}", path);
            if let Some(selected_idx) = state.playlists.selected {
                if selected_idx < state.playlists.playlists.len() {
                    let playlist_id = state.playlists.playlists[selected_idx].id;
                    match std::fs::canonicalize(&path) {
                        Ok(abs_path) => {
                            let path_str = abs_path.to_string_lossy().to_string();
                            let filename = abs_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            let extension = abs_path
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if ["mp3", "wav", "flac", "ogg", "m4a", "aac"].contains(&extension.as_str()) {
                                let track = core::Track {
                                    path: path_str,
                                    title: Some(filename),
                                    artist: None,
                                    album: None,
                                };
                                state.handle_action(core::Action::Playlist(core::PlaylistAction::AddTrack(playlist_id, track)));
                            } else {
                                log::warn!("Not a supported audio format: {}", extension);
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed to canonicalize dropped file: {}", e);
                        }
                    }
                }
            }
            Task::none()
        },
        Message::FilesHoveredLeft => {
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

fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    iced::event::listen().map(|event| match event {
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => match key {
            Key::Named(Named::Space) => {
                Message::Action(core::Action::Player(core::PlayerAction::Pause))
            }
            Key::Named(Named::Escape) => {
                Message::Action(core::Action::Player(core::PlayerAction::Stop))
            }
            _ => Message::Playlist(PlaylistAction::None),
        },
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Message::MousePosition(position)
        }
        iced::Event::Window(iced::window::Event::CloseRequested) => {
            Message::WindowClosed { x: 100, y: 100 }
        }
        iced::Event::Window(iced::window::Event::Moved(position)) => {
            let x = position.x as i32;
            let y = position.y as i32;
            Message::WindowClosed { x, y }
        }
        iced::Event::Window(iced::window::Event::Unfocused) => Message::WindowFocusLost,
        iced::Event::Window(iced::window::Event::Focused) => Message::WindowFocusGained,
        iced::Event::Window(iced::window::Event::FileHovered(_)) => Message::FileHovered,
        iced::Event::Window(iced::window::Event::FileDropped(path)) => {
            Message::FileDropped(path)
        }
        iced::Event::Window(iced::window::Event::FilesHoveredLeft) => Message::FilesHoveredLeft,
        _ => Message::Playlist(PlaylistAction::None),
    })
}

pub fn run() -> iced::Result {
    iced::application("Media Player", update, view)
        .subscription(subscription)
        .window(window_state::window_settings())
        .theme(|_state| ui::theme::dark_theme())
        .run()
}