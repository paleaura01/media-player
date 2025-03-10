use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ctrlc;
use std::error::Error;
use std::fs;

#[cfg(target_os = "windows")]
const DYLIB_NAME: &str = "player_ui.dll";
#[cfg(target_os = "linux")]
const DYLIB_NAME: &str = "libplayer_ui.so";
#[cfg(target_os = "macos")]
const DYLIB_NAME: &str = "libplayer_ui.dylib";

fn copy_ui_library() -> Result<(), Box<dyn Error>> {
    use std::path::Path;
    let source_path = Path::new("target").join("debug").join(DYLIB_NAME);
    let destination_path = Path::new("player_ui");
    
    if !source_path.exists() {
        eprintln!("UI library not found at {:?}", source_path);
        return Err("Source library not found".into());
    }
    
    // On Windows, handle file locking issues
    #[cfg(target_os = "windows")]
    {
        // Try several times with delays
        let max_attempts = 5;
        for attempt in 1..=max_attempts {
            match fs::copy(&source_path, destination_path) {
                Ok(_) => {
                    println!("Copied UI library from {:?} to {:?}", source_path, destination_path);
                    return Ok(());
                },
                Err(e) => {
                    if attempt < max_attempts {
                        println!("Copy attempt {} failed: {}. Retrying...", attempt, e);
                        std::thread::sleep(Duration::from_millis(300));
                    } else {
                        eprintln!("All copy attempts failed: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }
        // If we got here, all attempts failed
        return Err("Failed to copy library after multiple attempts".into());
    }
    
    // Non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    {
        fs::copy(&source_path, destination_path)?;
        println!("Copied UI library from {:?} to {:?}", source_path, destination_path);
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting development environment...");
    
    // Ensure directories exist
    for dir in &["./app/src/ui", "./assets", "./data"] {
        if !Path::new(dir).exists() {
            std::fs::create_dir_all(dir)?;
            println!("Created directory: {}", dir);
        }
    }
    
    // Create an empty playlists.json if it doesn't exist
    let playlists_path = Path::new("./data/playlists.json");
    if !playlists_path.exists() {
        std::fs::write(playlists_path, &[])?;
        println!("Created empty playlists.json");
    }
    
    // Build the library initially
    println!("Building UI library...");
    let status = Command::new("cargo")
        .args(["build", "--package", "app"])
        .status()?;
    
    if !status.success() {
        eprintln!("Failed to build UI library!");
        return Err("Build failed".into());
    }
    
    // Copy the built dynamic library to the project root
    copy_ui_library()?;
    
    // For breaking out of nested loops
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    
    // Set up the Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\nShutting down development environment...");
        r.store(false, Ordering::SeqCst);
    })?;
    
    let result = start_app_and_watch(running);
    
    println!("Development environment shutdown complete.");
    result
}

fn start_app_and_watch(running: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting main application...");
    let mut app_process = Command::new("cargo")
        .args(["run", "--bin", "media-player-app", "--package", "app"])
        .spawn()?;
    
    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx)?;
    
    // Watch the app/src/ui directory
    watcher.watch(Path::new("./app/src/ui"), RecursiveMode::Recursive)?;
    println!("Watching for changes in app/src/ui...");
    println!("Press Ctrl+C to stop");
    
    // Debounce mechanism
    let mut last_rebuild_time = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(1500);
    
    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if is_file_change(&event) && last_rebuild_time.elapsed() > debounce_duration {
                    last_rebuild_time = std::time::Instant::now();
                    
                    println!("\n\n===== UI CHANGE DETECTED =====");
                    for path in &event.paths {
                        if let Some(path_str) = path.to_str() {
                            println!("Changed file: {}", path_str);
                        }
                    }
                    
                    std::thread::sleep(Duration::from_millis(500));
                    
                    // Build ONLY the library component
                    println!("Rebuilding UI library...");
                    let rebuild_status = Command::new("cargo")
                        .args(["build", "--lib", "--package", "app"])
                        .status()?;
                        
                    if !rebuild_status.success() {
                        eprintln!("Rebuild failed, waiting for the next change...");
                        continue;
                    }
                    
                    // Try copying the library and restart the app if we can't copy
                    match copy_ui_library() {
                        Ok(_) => {
                            println!("Rebuilt UI library. The application needs to be restarted for changes to take effect.");
                            // Kill and restart the app
                            if let Some(child) = app_process.try_wait()? {
                                println!("Application exited with status: {}", child);
                            } else {
                                let _ = app_process.kill();
                                println!("Terminated application to apply changes");
                            }
                            app_process = Command::new("cargo")
                                .args(["run", "--bin", "media-player-app", "--package", "app"])
                                .spawn()?;
                        },
                        Err(e) => {
                            eprintln!("Failed to copy UI library: {}", e);
                        }
                    }
                }
            },
            Ok(Err(e)) => {
                eprintln!("Error receiving file event: {:?}", e);
            },
            Err(RecvTimeoutError::Timeout) => { /* continue */ },
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
                break;
            }
        }
        
        // Check if the app is still running
        match app_process.try_wait() {
            Ok(Some(status)) => {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                println!("Application exited with status: {}", status);
                println!("Restarting application...");
                app_process = Command::new("cargo")
                    .args(["run", "--bin", "media-player-app", "--package", "app"])
                    .spawn()?;
            },
            Ok(None) => {},
            Err(e) => {
                eprintln!("Error checking app status: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

fn is_file_change(event: &Event) -> bool {
    match event.kind {
        EventKind::Modify(_) => {
            event.paths.iter().any(|p| {
                p.extension().map_or(false, |ext| ext == "rs" || ext == "toml")
            })
        },
        _ => false,
    }
}
