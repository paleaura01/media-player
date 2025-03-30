// app/src/application.rs
use iced::{Element, Subscription, Task, Point};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::{Key, key::Named};
use std::path::PathBuf;
use std::time::Duration;

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
    /// Timer tick for background updates
    Tick,
    /// Clear seek flag (for async operations)
    ClearSeekFlag,
    /// Set a temporary status message
    SetStatusMessage(String, Duration),
    /// File validation result
    FileValidationResult(Option<(String, PathBuf, u32)>), // path_str, abs_path, playlist_id
}

fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    // Preserve shuffle between updates
    let shuffle_before = state.player_state.shuffle_enabled;

    // Clone message early to avoid borrowing issues
    let message_clone = message.clone();

    // Return a Tick task for any message that is Tick
    match &message {
        Message::Tick => {
            // Only update state on tick if we're not currently in a seek operation
            if !state.playlist_view_state.is_seeking {
                state.player.update_progress();
                state.player_state = state.player.get_state();
                state.player_state.shuffle_enabled = shuffle_before;
                state.check_for_completed_tracks();
            }
            
            // Clear status message if expired
            if let (Some(time), Some(duration)) = (state.status_message_time, state.status_message_duration) {
                if time.elapsed() > duration {
                    state.status_message = None;
                    state.status_message_time = None;
                    state.status_message_duration = None;
                }
            }
            
            // Schedule the next tick
            return Task::perform(
                async {
                    // Using async-std's sleep function
                    async_std::task::sleep(Duration::from_millis(100)).await;
                },
                |_| Message::Tick
            );
        },

        // Special handler for ClearSeekFlag message
        Message::ClearSeekFlag => {
            state.playlist_view_state.is_seeking = false;
            log::debug!("Cleared seeking flag asynchronously");
            
            // Continue ticking
            return Task::perform(
                async {
                    async_std::task::sleep(Duration::from_millis(100)).await;
                },
                |_| Message::Tick
            );
        },
        
        // Handle status message
        Message::SetStatusMessage(msg, duration) => {
            state.status_message = Some(msg.clone());
            state.status_message_time = Some(std::time::Instant::now());
            state.status_message_duration = Some(*duration);
            
            // Schedule a tick to clear the message after duration
            let duration_clone = *duration;
            return Task::perform(
                async move {
                    async_std::task::sleep(duration_clone).await;
                },
                |_| Message::Tick
            );
        },
        
        // Handle file validation result
        Message::FileValidationResult(result) => {
            if let Some((path_str, abs_path, playlist_id)) = result {
                let path_str_clone = path_str.clone();
                let abs_path_clone = abs_path.clone();
                let playlist_id_clone = *playlist_id;

                // Use FFmpeg to determine if file is supported audio
                if core::audio::decoder::is_supported_audio_format(&path_str_clone) {
                    let filename = abs_path_clone
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                        
                    let track = core::Track {
                        path: path_str_clone,
                        title: Some(filename.clone()),
                        artist: None,
                        album: None,
                        play_count: 0, // Initialize play count to 0
                    };
                    state.handle_action(core::Action::Playlist(core::PlaylistAction::AddTrack(playlist_id_clone, track)));
                    log::info!("Added audio file to playlist: {}", filename);
                    
                    return Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    );
                } else {
                    // Show a status message that the file is not supported
                    let path_str_for_closure = path_str_clone.clone();
                    return Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(1)).await;
                        },
                        move |_| Message::SetStatusMessage(format!("Unsupported audio format: {}", path_str_for_closure), Duration::from_secs(3))
                    );
                }
            } else {
                // File validation failed
                return Task::perform(
                    async {
                        async_std::task::sleep(Duration::from_millis(1)).await;
                    },
                    |_| Message::SetStatusMessage("File validation failed".to_string(), Duration::from_secs(3))
                );
            }
        },
        
        _ => {
            // For non-Tick messages, update state only if not in a seek operation
            if !matches!(message, 
                Message::Playlist(PlaylistAction::Seek(_)) | 
                Message::Playlist(PlaylistAction::UpdateProgress(_))
            ) {
                state.player.update_progress();
                state.player_state = state.player.get_state();
                state.player_state.shuffle_enabled = shuffle_before;
                state.check_for_completed_tracks();
            }
        }
    }

    // Process all other messages
    match message_clone {
        Message::Tick | Message::ClearSeekFlag | Message::SetStatusMessage(_, _) | Message::FileValidationResult(_) => {
            // Already handled above
            Task::none() 
        },
        Message::Action(action) => {
            let shuffle_before = state.player_state.shuffle_enabled;
            match action {
                core::Action::Player(player_action) => {
                    match player_action {
                        core::PlayerAction::Seek(pos) => {
                            // Set seeking flag
                            state.playlist_view_state.is_seeking = true;
                            
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
                            
                            // Clear seeking flag asynchronously after a delay
                            Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(50)).await;
                                },
                                |_| Message::ClearSeekFlag
                            )
                        },
                        core::PlayerAction::SetVolume(vol) => {
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                            
                            // Continue ticking
                            Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(100)).await;
                                },
                                |_| Message::Tick
                            )
                        },
                        core::PlayerAction::Shuffle => {
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                            
                            // Continue ticking
                            Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(100)).await;
                                },
                                |_| Message::Tick
                            )
                        },
                        core::PlayerAction::Play(ref path) => {
                            let is_network = path.starts_with("\\\\") || path.contains("://");
                            
                            // For network files, show a loading message
                            if is_network {
                                let _ = Task::perform(
                                    async { async_std::task::sleep(Duration::from_millis(1)).await; },
                                    |_| Message::SetStatusMessage("Loading network file...".to_string(), Duration::from_secs(3))
                                );
                            }
                            
                            // Handle the play action
                            state.handle_action(core::Action::Player(player_action));
                            state.player_state.shuffle_enabled = shuffle_before;
                            
                            // Continue ticking
                            Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(100)).await;
                                },
                                |_| Message::Tick
                            )
                        },
                        _ => {
                            state.handle_action(core::Action::Player(player_action));
                            state.player_state.shuffle_enabled = shuffle_before;
                            
                            // Continue ticking
                            Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(100)).await;
                                },
                                |_| Message::Tick
                            )
                        }
                    }
                },
                _ => {
                    state.handle_action(action);
                    state.player_state.shuffle_enabled = shuffle_before;
                    
                    // Continue ticking
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    )
                }
            }
        },
        Message::Playlist(action) => {
            match action {
                PlaylistAction::Seek(pos) => {
                    // Set seeking flag
                    state.playlist_view_state.is_seeking = true;
                    
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
                    
                    // Clear seeking flag asynchronously after a delay
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::ClearSeekFlag
                    )
                },
                PlaylistAction::UpdateProgress(pos) => {
                    // Set seeking flag during dragging too
                    state.playlist_view_state.is_seeking = true;
                    
                    // Just update the UI progress without initiating a seek
                    state.player_state.progress = pos;
                    log::debug!("Updated UI progress during dragging to {:.4}", pos);
                    
                    // Continue ticking
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    )
                },
                PlaylistAction::PlayTrack(pid, tid) => {
                    log::info!("Playing track {} from playlist {}", tid, pid);
                    
                    // Check if the track is from a network path and get its title
                    let track_info = state.playlists.get_playlist(pid)
                        .and_then(|p| p.tracks.get(tid))
                        .map(|t| (t.path.clone(), t.title.clone().unwrap_or_else(|| t.path.clone())));
                    
                    if let Some((path, title)) = track_info {
                        if path.starts_with("\\\\") || path.contains("://") {
                            let _ = Task::perform(
                                async { async_std::task::sleep(Duration::from_millis(1)).await; },
                                move |_| Message::SetStatusMessage(format!("Playing: {}", title), 
                                    Duration::from_secs(2))
                            );
                        }
                    }
                    
                    state.handle_action(core::Action::Playlist(core::PlaylistAction::PlayTrack(pid, tid)));
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                    
                    // Continue ticking
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    )
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
                            // Set seeking flag
                            state.playlist_view_state.is_seeking = true;
                            
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
                            
                            // Clear seeking flag asynchronously
                            return Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(100)).await;
                                },
                                |_| Message::ClearSeekFlag
                            );
                        },
                        _ => {
                            state.handle_action(core::Action::Player(ctrl));
                            state.player_state = state.player.get_state();
                            state.player_state.shuffle_enabled = shuffle_before;
                        }
                    }
                    
                    // Continue ticking
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    )
                },
                PlaylistAction::Library(library_action) => {
                    // Handle library messages forwarded through PlaylistAction
                    match library_action {
                        LibraryMessage::AddMusicFolder => {
                            // Open a folder selection dialog asynchronously
                            return Task::perform(
                                async {
                                    rfd::AsyncFileDialog::new().pick_folder().await.map(|f| f.path().to_owned())
                                },
                                Message::FolderSelected
                            );
                        },
                        LibraryMessage::ToggleView => {
                            // Handle toggle view if needed
                        }
                    }
                    
                    // Continue background ticks
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    )
                },
                _ => {
                    // other playlist actions
                    let core_action = state.playlist_view_state.handle_action(action);
                    state.handle_action(core_action);
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                    
                    // For non-seek operations, continue the background ticks
                    Task::perform(
                        async {
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::Tick
                    )
                }
            }
        },
        Message::Library(library_message) => {
            // Direct library messages
            match library_message {
                LibraryMessage::AddMusicFolder => {
                    // Open a folder selection dialog asynchronously
                    return Task::perform(
                        async {
                            rfd::AsyncFileDialog::new().pick_folder().await.map(|f| f.path().to_owned())
                        },
                        Message::FolderSelected
                    );
                },
                LibraryMessage::ToggleView => {
                    // Handle toggle view if needed
                }
            }
            
            // Continue background ticks
            Task::perform(
                async {
                    async_std::task::sleep(Duration::from_millis(100)).await;
                },
                |_| Message::Tick
            )
        },
        // All other message variants should return a background tick
        _ => {
            // Process the message as before...
            match message_clone {
                Message::FolderSelected(Some(path)) => {
                    if let Some(path_str) = path.to_str() {
                        log::info!("Selected music folder: {}", path_str);
                        
                        // Check if this is a network path
                        if path_str.starts_with("\\\\") || path_str.contains("://") {
                            // For network paths, show a status message
                            let _path_str_owned = path_str.to_owned();
                            let _ = Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(1)).await;
                                },
                                move |_| Message::SetStatusMessage("Scanning network folder...".to_string(), Duration::from_secs(3))
                            );
                        }
                        
                        state.handle_action(core::Action::Library(
                            core::LibraryAction::AddScanDirectory(path_str.to_string()),
                        ));
                        state.handle_action(core::Action::Library(core::LibraryAction::StartScan));
                    }
                },
                Message::FolderSelected(None) => {
                    log::info!("Folder selection cancelled");
                },
                Message::WindowClosed { x, y } => {
                    if let Err(e) = window_state::save_window_position(x, y) {
                        log::error!("Failed to save window position: {}", e);
                    }
                },
                Message::MousePosition(_pos) => {},
                Message::WindowFocusLost => {},
                Message::WindowFocusGained => {},
                Message::FileHovered => {},
                Message::FileDropped(path) => {
                    log::info!("File dropped: {:?}", path);
                    
                    if let Some(selected_idx) = state.playlists.selected {
                        if selected_idx < state.playlists.playlists.len() {
                            let playlist_id = state.playlists.playlists[selected_idx].id;
                            
                            // Show file processing message
                            let _ = Task::perform(
                                async {
                                    async_std::task::sleep(Duration::from_millis(1)).await;
                                },
                                |_| Message::SetStatusMessage("Processing file...".to_string(), Duration::from_secs(2))
                            );
                            
                            // Use async file processing to avoid blocking the UI
                            let path_clone = path.clone();
                            let playlist_id_clone = playlist_id;
                            
                            // Use Task::perform for asynchronous file handling
                            return Task::perform(
                                async move {
                                    // This runs in a background task
                                    match std::fs::canonicalize(&path_clone) {
                                        Ok(abs_path) => {
                                            let path_str = abs_path.to_string_lossy().to_string();
                                            Some((path_str, abs_path, playlist_id_clone))
                                        },
                                        _ => None,
                                    }
                                },
                                Message::FileValidationResult
                            );
                        }
                    }
                },
                Message::FilesHoveredLeft => {},
                _ => {},
            }
            
            // Always schedule a new tick for background updates
            Task::perform(
                async {
                    async_std::task::sleep(Duration::from_millis(100)).await;
                },
                |_| Message::Tick
            )
        }
    }
}

fn view(state: &MediaPlayer) -> Element<Message> {
    // Get our PlaylistAction element from render_with_state
    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        &state.playlist_view_state,
        &state.status_message,
    );
    
    // Check if it's a Library action and map accordingly
    match rendered {
        playlist_element => {
            playlist_element.map(|action| {
                match action {
                    PlaylistAction::Library(lib_msg) => Message::Library(lib_msg),
                    other => Message::Playlist(other),
                }
            })
        }
    }
}

fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    // Just listen to window events - we'll use Task::perform for the timer
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
        iced::Event::Window(iced::window::Event::RedrawRequested(_)) => {
            // Just use this to get the first tick when the app starts
            Message::Tick
        },
        _ => Message::Playlist(PlaylistAction::None),
    })
}

pub fn run() -> iced::Result {
    // Initialize FFmpeg at application start
    if let Err(e) = core::audio::decoder::initialize_ffmpeg() {
        log::error!("Failed to initialize FFmpeg: {}", e);
    } else {
        log::info!("FFmpeg initialized successfully");
        
        // Log supported formats
        let formats = core::audio::decoder::get_supported_extensions();
        log::info!("Supported audio formats: {}", formats.join(", "));
    }
    
    iced::application("Media Player", update, view)
        .subscription(subscription)
        .window(window_state::window_settings())
        .theme(|_state| ui::theme::dark_theme())
        .run()
}