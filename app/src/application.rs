// app/src/application.rs - Add improved batch processing support
use iced::{Element, Subscription, Task};
use crate::ui;
use crate::states::window_state;
use crate::states::app_state::MediaPlayer;
use iced::keyboard::{Key, key::Named};
use std::path::PathBuf;
use std::time::Duration;
use std::fs;
use tokio::time::sleep; // Use tokio instead

use crate::ui::playlist_view::PlaylistAction;
use crate::ui::library_view::LibraryMessage;

// Create a structure to represent a batch processing job - make it public
#[derive(Debug, Clone)]
pub struct BatchProcessingJob {
    files: Vec<PathBuf>,
    playlist_id: u32,
    current_index: usize,
    batch_size: usize,
    processed_count: usize,
    failed_count: usize,
}

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
    MousePosition(()),
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
    FileValidationResult(Option<(String, PathBuf, u32)>),
    /// Batch file processing result
    BatchProcessingResult(Vec<(String, PathBuf)>, u32),
    /// Process next batch of files
    ProcessNextBatch(BatchProcessingJob),
    /// Batch processing complete notification
    BatchProcessingComplete(usize, usize, u32),
    /// Directory scan result for batch processing
    DirectoryScanResult(Vec<PathBuf>, u32),
}

// Main update function - ensures every arm returns Task<Message>
fn update(state: &mut MediaPlayer, message: Message) -> Task<Message> {
    // Preserve shuffle between updates
    let shuffle_before = state.player_state.shuffle_enabled;

    // Clone message early to avoid borrowing issues
    let message_clone = message.clone();

    // The main match expression determines the return value
    match message_clone {
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
            Task::perform(
                async {
                    sleep(Duration::from_millis(100)).await;
                },
                |_| Message::Tick
            )
        },

        Message::ClearSeekFlag => {
            state.playlist_view_state.is_seeking = false;
            log::debug!("Cleared seeking flag asynchronously");

            // Continue ticking
            Task::perform(
                async {
                    sleep(Duration::from_millis(100)).await;
                },
                |_| Message::Tick
            )
        },

        Message::SetStatusMessage(msg, duration) => {
            state.status_message = Some(msg.clone());
            state.status_message_time = Some(std::time::Instant::now());
            state.status_message_duration = Some(duration);

            // Schedule a tick to clear the message after duration
            let duration_clone = duration;
            Task::perform(
                async move {
                    sleep(duration_clone).await;
                },
                |_| Message::Tick
            )
        },

        Message::FileValidationResult(result) => {
            if let Some((path_str, abs_path, playlist_id)) = result {
                let path_str_clone = path_str.clone();
                let abs_path_clone = abs_path.clone();
                let playlist_id_clone = playlist_id;

                if core::audio::decoder::is_supported_audio_format(&path_str_clone) {
                    let filename = abs_path_clone
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let track = core::Track {
                        path: path_str_clone.clone(),
                        title: Some(filename.clone()),
                        artist: None,
                        album: None,
                        play_count: 0,
                    };
                    state.handle_action(core::Action::Playlist(core::PlaylistAction::AddTrack(playlist_id_clone, track)));
                    
                    // Force save after adding track
                    if let Err(e) = state.save_playlists() {
                        log::error!("Failed to save playlist after adding track: {}", e);
                    }
                    
                    log::info!("Added audio file to playlist: {}", filename);

                    Task::perform(
                        async { sleep(Duration::from_millis(1)).await; },
                        move |_| Message::SetStatusMessage(format!("Added: {}", filename), Duration::from_secs(2))
                    )
                } else {
                    let path_str_for_closure = path_str_clone.clone();
                    Task::perform(
                        async { sleep(Duration::from_millis(1)).await; },
                        move |_| Message::SetStatusMessage(format!("Unsupported: {}", path_str_for_closure), Duration::from_secs(3))
                    )
                }
            } else {
                Task::perform(
                    async { sleep(Duration::from_millis(1)).await; },
                    |_| Message::SetStatusMessage("File validation failed".to_string(), Duration::from_secs(3))
                )
            }
        },

        Message::DirectoryScanResult(files, playlist_id) => {
            if files.is_empty() {
                return Task::perform(
                    async { sleep(Duration::from_millis(1)).await; },
                    |_| Message::SetStatusMessage("No files found in directory".to_string(), Duration::from_secs(2))
                );
            }

            log::info!("Found {} files to process for playlist {}", files.len(), playlist_id);

            // Create a BatchProcessingJob with even smaller batch size for stability
            let job = BatchProcessingJob {
                files: files.clone(),
                playlist_id,
                current_index: 0,
                batch_size: 15, // REDUCED MORE FROM 25 to 15 for better stability
                processed_count: 0,
                failed_count: 0,
            };

            // Set the batch processing flag
            state.is_batch_processing = true;
            
            let job_for_closure = job.clone();
            Task::perform(
                async { sleep(Duration::from_millis(250)).await; }, // INCREASED INITIAL DELAY
                move |_| Message::ProcessNextBatch(job_for_closure.clone())
            )
        },

        Message::ProcessNextBatch(job) => {
            let mut job_clone = job.clone();

            log::info!("BATCH: Processing batch {}/{} files - progress: {}/{}",
                      job_clone.current_index, job_clone.current_index + job_clone.batch_size,
                      job_clone.processed_count, job_clone.files.len());

            // CHECK IF SHOULD PAUSE PROCESSING
            if state.is_batch_processing && job_clone.current_index > 0 && job_clone.current_index % 100 == 0 {
                // Save playlists and take a longer break every 100 files
                if let Err(e) = state.save_playlists() {
                    log::error!("Failed to save playlists at checkpoint: {}", e);
                } else {
                    log::info!("BATCH CHECKPOINT: Saved playlist at {} files", job_clone.current_index);
                }
                
                // Take a longer pause every 100 files to let the system catch up
                return Task::perform(
                    async { sleep(Duration::from_secs(2)).await; }, // 2 SECOND PAUSE every 100 files
                    move |_| Message::ProcessNextBatch(job_clone.clone())
                );
            }

            if job_clone.current_index >= job_clone.files.len() {
                log::info!("BATCH: All batches completed. Processed {} files, failed {} files",
                          job_clone.processed_count, job_clone.failed_count);

                let processed = job_clone.processed_count;
                let failed = job_clone.failed_count;
                let playlist_id = job_clone.playlist_id;

                // Only save playlists once at the end of batch processing
                if let Err(e) = state.save_playlists() {
                    log::error!("Failed to save playlists after batch completion: {}", e);
                } else {
                    log::info!("BATCH: Final playlist save successful");
                }
                
                // Reset batch processing flag
                state.is_batch_processing = false;

                return Task::perform(
                    async { sleep(Duration::from_millis(500)).await; },
                    move |_| Message::BatchProcessingComplete(processed, failed, playlist_id)
                );
            }

            let end_index = std::cmp::min(job_clone.current_index + job_clone.batch_size, job_clone.files.len());
            
            let mut tracks = Vec::new();
            let playlist_id = job_clone.playlist_id;
            let mut _processed_in_batch = 0; // Added underscore to fix warning
            let mut _failed_in_batch = 0;    // Added underscore to fix warning

            for i in job_clone.current_index..end_index {
                if let Some(file_path) = job_clone.files.get(i) {
                    match std::fs::canonicalize(file_path) {
                        Ok(abs_path) => {
                            let path_str = abs_path.to_string_lossy().to_string();

                            if core::audio::decoder::is_supported_audio_format(&path_str) {
                                let filename = abs_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown")
                                    .to_string();

                                tracks.push(core::Track {
                                    path: path_str,
                                    title: Some(filename),
                                    artist: None,
                                    album: None,
                                    play_count: 0,
                                });

                                job_clone.processed_count += 1;
                                _processed_in_batch += 1;  // Updated with underscore
                            } else {
                                job_clone.failed_count += 1;
                                _failed_in_batch += 1;     // Updated with underscore
                            }
                        },
                        Err(_) => {
                            job_clone.failed_count += 1;
                            _failed_in_batch += 1;         // Updated with underscore
                        }
                    }
                }
            }

            job_clone.current_index = end_index;

            if !tracks.is_empty() {
                state.handle_action(core::Action::Playlist(
                    core::PlaylistAction::BatchAddTracks(playlist_id, tracks)
                ));
                
                // Save every 200 files
                if job_clone.current_index % 200 == 0 || job_clone.current_index >= job_clone.files.len() {
                    if let Err(e) = state.save_playlists() {
                        log::error!("Failed to save playlists after batch: {}", e);
                    }
                }
            }

            // Schedule next batch with much longer delays
            let job_for_next_batch = job_clone.clone();
            
            // SIGNIFICANTLY INCREASED DELAY between batches
            let total_files = job_clone.files.len() as f32;
            let progress = job_clone.current_index as f32 / total_files;
            
            // Progressively increase delay as we process more files
            // Start at 500ms at the beginning and increase up to 1.5 seconds
            let delay_ms = 500 + ((progress * 1000.0) as u64);
            
            log::info!("QUEUE: Scheduling next batch with {} ms delay...", delay_ms);
            
            Task::perform(
                async move {
                    sleep(Duration::from_millis(delay_ms)).await;
                },
                move |_| Message::ProcessNextBatch(job_for_next_batch.clone())
            )
        },

        Message::BatchProcessingComplete(processed, failed, _playlist_id) => {
            log::info!("Processed {} files successfully, {} files failed", processed, failed);
            
            // Force final save
            if let Err(e) = state.save_playlists() {
                log::error!("Failed to save playlists after batch completion: {}", e);
            } else {
                log::info!("BATCH: Final playlist save successful after batch processing");
            }
            
            Task::perform(
                async { sleep(Duration::from_millis(500)).await; },
                move |_| Message::SetStatusMessage(
                    format!("Added {} audio files ({} failed)", processed, failed),
                    Duration::from_secs(3)
                )
            )
        },

        Message::BatchProcessingResult(files, playlist_id) => {
            if files.is_empty() {
                return Task::perform(
                    async { sleep(Duration::from_millis(1)).await; },
                    |_| Message::SetStatusMessage("No valid files to add".to_string(), Duration::from_secs(2))
                );
            }

            let mut tracks = Vec::with_capacity(files.len());
            let mut supported_count = 0;

            for (path_str, abs_path) in files {
                if core::audio::decoder::is_supported_audio_format(&path_str) {
                    let filename = abs_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    tracks.push(core::Track {
                        path: path_str.clone(),
                        title: Some(filename),
                        artist: None,
                        album: None,
                        play_count: 0,
                    });

                    supported_count += 1;
                }
            }

            if !tracks.is_empty() {
                state.handle_action(core::Action::Playlist(
                    core::PlaylistAction::BatchAddTracks(playlist_id, tracks)
                ));
                
                if let Err(e) = state.save_playlists() {
                    log::error!("Failed to save playlists after batch result: {}", e);
                }
                
                let final_count = supported_count;
                Task::perform(
                    async { sleep(Duration::from_millis(1)).await; },
                    move |_| Message::SetStatusMessage(format!("Added {} files to playlist", final_count), Duration::from_secs(2))
                )
            } else {
                Task::perform(
                    async { sleep(Duration::from_millis(1)).await; },
                    |_| Message::SetStatusMessage("No supported audio files found".to_string(), Duration::from_secs(3))
                )
            }
        },

        // --- Action Handlers ---
        Message::Action(action) => {
            let shuffle_before = state.player_state.shuffle_enabled;
            match action {
                core::Action::Player(player_action) => {
                    match player_action {
                        core::PlayerAction::Seek(pos) => {
                            state.playlist_view_state.is_seeking = true;
                            state.player.clear_audio_buffers();
                            if let Ok(mut lock) = state.player.playback_position.lock() {
                                lock.request_seek(pos);
                                log::debug!("(Action) Requested seek to {:.4}", pos);
                            } else {
                                log::error!("(Action) Failed to acquire lock for seek to {:.4}", pos);
                            }
                            state.player_state.progress = pos;
                            Task::perform(
                                async { sleep(Duration::from_millis(50)).await; },
                                |_| Message::ClearSeekFlag
                            )
                        },
                        core::PlayerAction::SetVolume(vol) => {
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                            Task::none() // Volume change is synchronous
                        },
                        core::PlayerAction::Shuffle => {
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                            Task::none() // Shuffle toggle is synchronous
                        },
                        core::PlayerAction::Play(ref path) => {
                            let is_network = path.starts_with("\\\\") || path.contains("://");
                            state.handle_action(core::Action::Player(player_action));
                            state.player_state.shuffle_enabled = shuffle_before;
                            if is_network {
                                Task::perform(
                                    async { sleep(Duration::from_millis(1)).await; },
                                    |_| Message::SetStatusMessage("Loading network file...".to_string(), Duration::from_secs(3))
                                )
                            } else {
                                Task::none() // Playback starts in a thread, no immediate task needed here
                            }
                        },
                        _ => {
                            state.handle_action(core::Action::Player(player_action));
                            state.player_state.shuffle_enabled = shuffle_before;
                            Task::none() // Other player actions are synchronous or handled by playback thread
                        }
                    }
                },
                _ => { // Playlist or Library Actions passed via Action enum
                    state.handle_action(action);
                    state.player_state.shuffle_enabled = shuffle_before;
                    
                    // Only save if not in batch processing
                    if !state.is_batch_processing {
                        if let Err(e) = state.save_playlists() {
                            log::error!("Failed to save playlists after action: {}", e);
                        }
                    }
                    
                    Task::none() // Playlist/Library actions are typically synchronous state updates
                }
            }
        },

        Message::Playlist(action) => {
            match action {
                PlaylistAction::Seek(pos) => {
                    state.playlist_view_state.is_seeking = true;
                    log::info!("(Playlist) Received Seek({:.4})", pos);
                    state.player.clear_audio_buffers();
                    if let Ok(mut lock) = state.player.playback_position.lock() {
                        lock.request_seek(pos);
                        log::debug!("Seek request set for {:.4}", pos);
                    } else {
                        log::error!("Failed to acquire lock for seek request");
                    }
                    state.player_state.progress = pos;
                    Task::perform(
                        async { sleep(Duration::from_millis(100)).await; },
                        |_| Message::ClearSeekFlag
                    )
                },
                PlaylistAction::UpdateProgress(pos) => {
                    state.playlist_view_state.is_seeking = true;
                    state.player_state.progress = pos;
                    log::debug!("Updated UI progress during dragging to {:.4}", pos);
                    Task::none() // Just UI update, no task needed
                },
                PlaylistAction::PlayTrack(pid, tid) => {
                    log::info!("Playing track {} from playlist {}", tid, pid);
                    let path_str = state.playlists.get_playlist(pid)
                        .and_then(|p| p.tracks.get(tid))
                        .map(|t| t.path.clone());
                    state.handle_action(core::Action::Playlist(core::PlaylistAction::PlayTrack(pid, tid)));
                    state.player_state = state.player.get_state(); // Update state after handle_action
                    state.player_state.shuffle_enabled = shuffle_before;
                    if let Some(path) = path_str {
                        if path.starts_with("\\\\") || path.contains("://") {
                           Task::perform(
                                async { sleep(Duration::from_millis(1)).await; },
                                |_| Message::SetStatusMessage("Loading network file...".to_string(), Duration::from_secs(3))
                            )
                        } else {
                            Task::none()
                        }
                    } else {
                        Task::none()
                    }
                },
                PlaylistAction::BatchAddTracks(pid, tracks) => {
                    log::info!("Adding {} tracks to playlist {} in batch", tracks.len(), pid);
                    state.handle_action(core::Action::Playlist(core::PlaylistAction::BatchAddTracks(pid, tracks)));
                    
                    // Only save if not in batch processing mode
                    if !state.is_batch_processing {
                        // Force save after batch add
                        if let Err(e) = state.save_playlists() {
                            log::error!("Failed to save playlists after batch add: {}", e);
                        } else {
                            log::info!("Successfully saved playlists after batch add");
                        }
                    }
                    
                    Task::none()
                },
                PlaylistAction::PlayerControl(ctrl) => {
                    match ctrl {
                        core::PlayerAction::SetVolume(vol) => {
                            state.player.set_volume(vol);
                            state.player_state.volume = vol;
                            Task::none()
                        },
                        core::PlayerAction::Shuffle => {
                            state.player_state.shuffle_enabled = !state.player_state.shuffle_enabled;
                            Task::none()
                        },
                        core::PlayerAction::Seek(pos) => {
                            state.playlist_view_state.is_seeking = true;
                            state.player.clear_audio_buffers();
                            if let Ok(mut lock) = state.player.playback_position.lock() {
                                lock.request_seek(pos);
                                log::debug!("(PlayerControl) Seek request set for {:.4}", pos);
                            } else {
                                log::error!("(PlayerControl) Failed to acquire lock for seek request");
                            }
                            state.player_state.progress = pos;
                            Task::perform(
                                async { sleep(Duration::from_millis(100)).await; },
                                |_| Message::ClearSeekFlag
                            )
                        },
                        _ => {
                            state.handle_action(core::Action::Player(ctrl));
                            state.player_state = state.player.get_state();
                            state.player_state.shuffle_enabled = shuffle_before;
                            Task::none()
                        }
                    }
                },
                PlaylistAction::Library(library_action) => {
                    match library_action {
                        LibraryMessage::AddMusicFolder => {
                            Task::perform(
                                async { rfd::AsyncFileDialog::new().pick_folder().await.map(|f| f.path().to_owned()) },
                                Message::FolderSelected
                            )
                        },
                        LibraryMessage::ToggleView => {
                            // Handle toggle view if needed in state
                            Task::none()
                        }
                    }
                },
                _ => { // Other playlist actions like Select, Delete, Rename etc.
                    let core_action = state.playlist_view_state.handle_action(action);
                    state.handle_action(core_action);
                    state.player_state = state.player.get_state();
                    state.player_state.shuffle_enabled = shuffle_before;
                    
                    // Only save if not in batch processing
                    if !state.is_batch_processing {
                        // Force save after any playlist action
                        if let Err(e) = state.save_playlists() {
                            log::error!("Failed to save playlists after action: {}", e);
                        }
                    }
                    
                    Task::none() // These are synchronous state updates
                }
            }
        },

        Message::Library(library_message) => {
            match library_message {
                LibraryMessage::AddMusicFolder => {
                    Task::perform(
                        async { rfd::AsyncFileDialog::new().pick_folder().await.map(|f| f.path().to_owned()) },
                        Message::FolderSelected
                    )
                },
                LibraryMessage::ToggleView => {
                    // Handle toggle view if needed in state
                    Task::none()
                }
            }
        },

        Message::FolderSelected(Some(path)) => {
            if let Some(path_str) = path.to_str() {
                log::info!("Selected music folder: {}", path_str);
                state.handle_action(core::Action::Library(
                    core::LibraryAction::AddScanDirectory(path_str.to_string()),
                ));
                state.handle_action(core::Action::Library(core::LibraryAction::StartScan));

                // Schedule DirectoryScanResult Task after adding dir and starting scan
                 if path.is_dir() {
                    log::info!("Scanning directory for audio files recursively: {}", path_str);
                    let playlist_id = state.playlists.playlists.last().map(|p| p.id).unwrap_or(0); // Use last selected/created or default
                    let path_clone = path.clone();
                    Task::perform(
                        async move {
                            let mut files = Vec::new();
                            scan_directory_recursively(&path_clone, &mut files);
                            files
                        },
                        move |files| Message::DirectoryScanResult(files, playlist_id)
                    )
                } else {
                    Task::none() // Not a directory
                }

            } else {
                Task::none() // Path conversion failed
            }
        },
        Message::FolderSelected(None) => {
            log::info!("Folder selection cancelled");
            Task::none()
        },

        Message::WindowClosed { x, y } => {
            if let Err(e) = window_state::save_window_position(x, y) {
                log::error!("Failed to save window position: {}", e);
            }
            Task::none()
        },
        Message::MousePosition(_) => { Task::none() },
        Message::WindowFocusLost => { Task::none() },
        Message::WindowFocusGained => { Task::none() },
        Message::FileHovered => { Task::none() },

        Message::FileDropped(path) => {
            log::info!("File dropped: {:?}", path);
            if let Some(selected_idx) = state.playlists.selected {
                if selected_idx < state.playlists.playlists.len() {
                    let playlist_id = state.playlists.playlists[selected_idx].id;
                    
                    // Added logging for playlist ID tracking
                    log::info!("Using playlist ID {} for dropped files", playlist_id);

                    // Task for status message
                    let _status_task = Task::perform(
                        async { sleep(Duration::from_millis(1)).await; },
                        |_| Message::SetStatusMessage("Processing dropped item...".to_string(), Duration::from_secs(2))
                    );

                    if path.is_dir() {
                        log::info!("Found directory, scanning for audio files recursively");
                        let playlist_id_clone = playlist_id;
                        let path_clone = path.clone();
                        
                        // Use a separate thread for directory scanning to avoid UI blocking
                        Task::perform(
                            async move {
                                let mut files = Vec::new();
                                scan_directory_recursively(&path_clone, &mut files);
                                files
                            },
                            move |files| Message::DirectoryScanResult(files, playlist_id_clone)
                        )
                    } else {
                        let path_clone = path.clone();
                        let playlist_id_clone = playlist_id;
                        Task::perform(
                            async move {
                                match std::fs::canonicalize(&path_clone) {
                                    Ok(abs_path) => {
                                        let path_str = abs_path.to_string_lossy().to_string();
                                        Some((path_str, abs_path, playlist_id_clone))
                                    },
                                    _ => None,
                                }
                            },
                            Message::FileValidationResult
                        )
                    }
                } else {
                     Task::perform( // No valid playlist selected
                        async { sleep(Duration::from_millis(1)).await; },
                        |_| Message::SetStatusMessage("Select a playlist before dropping files".to_string(), Duration::from_secs(3))
                    )
                }
            } else {
                 Task::perform( // No playlist selected, show message
                    async { sleep(Duration::from_millis(1)).await; },
                    |_| Message::SetStatusMessage("Select a playlist before dropping files".to_string(), Duration::from_secs(3))
                )
            }
        },
        Message::FilesHoveredLeft => { Task::none() },
    } // End main match
}

// Recursive directory scanner function
fn scan_directory_recursively(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    log::info!("SCAN: Scanning directory: {:?}", dir);

    let count_before = files.len();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                scan_directory_recursively(&path, files);
            } else if path.is_file() {
                files.push(path.clone());
            }
        }
    }

    let count_after = files.len();
    log::info!("SCAN: Directory {:?} added {} files (total now: {})",
               dir, count_after - count_before, count_after);
}

fn view(state: &MediaPlayer) -> Element<Message> {
    let rendered = ui::render::render_with_state(
        &state.player_state,
        &state.playlists,
        &state.library,
        &state.playlist_view_state,
        &state.status_message,
    );

    // Map PlaylistAction to Message
    rendered.map(|action| {
        match action {
            PlaylistAction::Library(lib_msg) => Message::Library(lib_msg),
            other => Message::Playlist(other),
        }
    })
}

fn subscription(_state: &MediaPlayer) -> Subscription<Message> {
    // Listen to window events and keyboard, map them to Messages
    iced::event::listen().map(|event| match event {
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => match key {
            Key::Named(Named::Space) => {
                Message::Action(core::Action::Player(core::PlayerAction::Pause)) // Should toggle pause/resume
            }
            Key::Named(Named::Escape) => {
                Message::Action(core::Action::Player(core::PlayerAction::Stop))
            }
            _ => Message::Playlist(PlaylistAction::None), // Or a specific NoOp Message variant if preferred
        },
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position: _ }) => {
            Message::MousePosition(())
        }
        iced::Event::Window(iced::window::Event::CloseRequested) => {
            // Ideally get actual position here if possible, otherwise use last known or default
            Message::WindowClosed { x: 100, y: 100 }
        }
        iced::Event::Window(iced::window::Event::Moved(position)) => {
             // Save position on move as well
            Message::WindowClosed { x: position.x as i32, y: position.y as i32 }
        }
        iced::Event::Window(iced::window::Event::Unfocused) => Message::WindowFocusLost,
        iced::Event::Window(iced::window::Event::Focused) => Message::WindowFocusGained,
        iced::Event::Window(iced::window::Event::FileHovered(_)) => Message::FileHovered,
        // Use the corrected singular event name
        iced::Event::Window(iced::window::Event::FileDropped(path)) => {
            Message::FileDropped(path) // Map to our message
        }
        iced::Event::Window(iced::window::Event::FilesHoveredLeft) => Message::FilesHoveredLeft,
        // Use RedrawRequested for the initial Tick to start the timer loop
        iced::Event::Window(iced::window::Event::RedrawRequested(_)) => {
            Message::Tick
        },
        _ => Message::Playlist(PlaylistAction::None), // Catch-all for other events
    })
}

pub fn run() -> iced::Result {
    if let Err(e) = core::audio::decoder::initialize_ffmpeg() {
        log::error!("Failed to initialize FFmpeg: {}", e);
        // Optionally, you might want to return an error or show a message here
    } else {
        log::info!("FFmpeg initialized successfully");
        let formats = core::audio::decoder::get_supported_extensions();
        log::info!("Supported audio formats: {}", formats.join(", "));
    }

    iced::application("Media Player", update, view)
        .subscription(subscription)
        .window(window_state::window_settings())
        .theme(|_state| ui::theme::dark_theme())
        .run()
}