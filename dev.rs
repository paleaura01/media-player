use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::env;
use ctrlc;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting development environment...");
    
    // Ensure directories exist
    for dir in &["./app/src/ui", "./assets", "./data"] {
        if !Path::new(dir).exists() {
            std::fs::create_dir_all(dir)?;
            println!("Created directory: {}", dir);
        }
    }
    
    // Create an empty playlists.bin if it doesn't exist
    let playlists_path = Path::new("./data/playlists.json");
    if !playlists_path.exists() {
        std::fs::write(playlists_path, &[])?;
        println!("Created empty playlists.json");
    }
    
    // Determine library paths
    let lib_path = if cfg!(target_os = "windows") {
        "./target/debug/player_ui.dll"
    } else if cfg!(target_os = "macos") {
        "./target/debug/libplayer_ui.dylib"
    } else {
        "./target/debug/libplayer_ui.so"
    };
    
    // Create a hot reload copy path
    let hot_copy_path = if cfg!(target_os = "windows") {
        "./target/debug/player_ui-latest.dll"
    } else if cfg!(target_os = "macos") {
        "./target/debug/libplayer_ui-latest.dylib"
    } else {
        "./target/debug/libplayer_ui-latest.so"
    };
    
    // Set environment variable for hot-reloading to use the copy path
    env::set_var("HOT_LIB_RELOADER_LIBRARY_PATH", hot_copy_path);
    println!("Hot-reload path set to: {}", hot_copy_path);
    
    // Build library initially
    println!("Building UI library...");
    let status = Command::new("cargo")
        .args(["build", "--lib", "--package", "app"])
        .status()?;
    
    if !status.success() {
        eprintln!("Failed to build UI library!");
        return Err("Build failed".into());
    }
    
    // Create initial hot reload copy
    if Path::new(lib_path).exists() {
        // First try to remove existing copy if present
        if Path::new(hot_copy_path).exists() {
            let _ = fs::remove_file(hot_copy_path);
            std::thread::sleep(Duration::from_millis(100));
        }
        
        // Copy the library file
        match fs::copy(lib_path, hot_copy_path) {
            Ok(_) => println!("Created initial hot reload copy of library"),
            Err(e) => eprintln!("Failed to create hot reload copy: {}", e),
        }
    }
    
    // Start the main application
    println!("Starting main application...");
    let mut app_process = Command::new("cargo")
        .args(["run", "--bin", "media-player-app", "--package", "app"])
        .spawn()?;
    
    // Store the process ID for the Ctrl+C handler
    let app_process_id = app_process.id();
    
    // Give the app time to initialize before watching for changes
    std::thread::sleep(Duration::from_millis(2000));
    
    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx)?;
    
    // Watch the app/src directory
    watcher.watch(Path::new("./app/src"), RecursiveMode::Recursive)?;
    println!("Watching for changes in app/src...");
    println!("Press Ctrl+C to stop");
    
    // Main loop with improved Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\nShutting down development environment...");
        
        // Properly terminate the app process first
        if let Err(e) = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &app_process_id.to_string()])
            .status() {
            eprintln!("Failed to kill app process: {}", e);
        }
        
        // Give processes time to clean up
        std::thread::sleep(Duration::from_millis(500));
        
        // Exit cleanly
        std::process::exit(0);
    })?;
    
    // Debounce mechanism to prevent multiple rebuilds for a single change
    let mut last_rebuild_time = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(1000);
    
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if is_file_change(&event) && last_rebuild_time.elapsed() > debounce_duration {
                    last_rebuild_time = std::time::Instant::now();
                    
                    println!("\n\n===== CHANGE DETECTED =====");
                    
                    // Show which file was changed
                    for path in &event.paths {
                        if let Some(path_str) = path.to_str() {
                            println!("Changed file: {}", path_str);
                        }
                    }
                    
                    // Wait for file system to stabilize
                    std::thread::sleep(Duration::from_millis(500));
                    
                    // Perform a clean build
                    println!("Building UI library...");
                    
                    // Use a temporary file path to avoid conflicts
                    let temp_copy_path = if cfg!(target_os = "windows") {
                        "./target/debug/player_ui-temp.dll"
                    } else if cfg!(target_os = "macos") {
                        "./target/debug/libplayer_ui-temp.dylib"
                    } else {
                        "./target/debug/libplayer_ui-temp.so"
                    };
                    
                    let rebuild_success = {
                        let status = Command::new("cargo")
                            .args(["build", "--lib", "--package", "app"])
                            .status()?;
                        status.success()
                    };
                    
                    if rebuild_success && Path::new(lib_path).exists() {
                        println!("Build successful, updating hot reload copy...");
                        
                        // First create a temp copy
                        match fs::copy(lib_path, temp_copy_path) {
                            Ok(_) => {
                                // Then wait to make sure the app isn't actively using the file
                                std::thread::sleep(Duration::from_millis(500));
                                
                                // Swap the files
                                if Path::new(hot_copy_path).exists() {
                                    let _ = fs::remove_file(hot_copy_path);
                                }
                                
                                // Wait again for the file system
                                std::thread::sleep(Duration::from_millis(500));
                                
                                // Move the temp file to the hot reload path
                                match fs::rename(temp_copy_path, hot_copy_path) {
                                    Ok(_) => println!("UI UPDATED! Hot reload will pick up changes."),
                                    Err(e) => {
                                        eprintln!("Failed to rename library: {}", e);
                                        // Try copy instead if rename fails
                                        if let Ok(_) = fs::copy(temp_copy_path, hot_copy_path) {
                                            println!("UI UPDATED via copy instead of rename.");
                                            let _ = fs::remove_file(temp_copy_path);
                                        }
                                    }
                                }
                            },
                            Err(e) => eprintln!("Failed to create temp copy: {}", e),
                        }
                    } else {
                        eprintln!("Build failed or library file not found.");
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
                println!("Application exited with status: {}", status);
                break;
            },
            Ok(None) => { /* still running */ },
            Err(e) => {
                eprintln!("Error checking app status: {}", e);
                break;
            }
        }
    }
    
    // Clean up
    println!("Attempting to clean up application process.");
    if let Err(e) = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &app_process.id().to_string()])
        .status() {
        eprintln!("Failed to kill app process: {}", e);
    }
    
    Ok(())
}

fn is_file_change(event: &Event) -> bool {
    match event.kind {
        EventKind::Modify(_) => {
            event.paths.iter().any(|p| {
                p.extension().map_or(false, |ext| {
                    ext == "rs" || ext == "toml"
                })
            })
        },
        _ => false,
    }
}