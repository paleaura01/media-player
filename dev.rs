use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::env;
use ctrlc;

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
        .args(["build", "--lib", "--package", "app"])
        .status()?;
    
    if !status.success() {
        eprintln!("Failed to build UI library!");
        return Err("Build failed".into());
    }
    
    // Start the main application
    start_app_and_watch()?;
    
    Ok(())
}

fn start_app_and_watch() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting main application...");
    let mut app_process = Command::new("cargo")
        .args(["run", "--bin", "media-player-app", "--package", "app"])
        .spawn()?;
    
    // Store the process ID for the Ctrl+C handler
    let app_process_id = app_process.id();
    
    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx)?;
    
    // Watch the app/src/ui directory specifically
    watcher.watch(Path::new("./app/src/ui"), RecursiveMode::Recursive)?;
    println!("Watching for changes in app/src/ui...");
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
        std::thread::sleep(Duration::from_millis(1000));
        
        // Exit cleanly
        std::process::exit(0);
    })?;
    
    // Debounce mechanism to prevent multiple rebuilds for a single change
    let mut last_rebuild_time = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(1500);
    
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if is_file_change(&event) && last_rebuild_time.elapsed() > debounce_duration {
                    last_rebuild_time = std::time::Instant::now();
                    
                    println!("\n\n===== UI CHANGE DETECTED =====");
                    
                    // Show which file was changed
                    for path in &event.paths {
                        if let Some(path_str) = path.to_str() {
                            println!("Changed file: {}", path_str);
                        }
                    }
                    
                    // Wait for file system to stabilize
                    std::thread::sleep(Duration::from_millis(500));
                    
                    // Terminate the app
                    println!("Terminating app to reload changes...");
                    if let Err(e) = Command::new("taskkill")
                        .args(["/F", "/T", "/PID", &app_process_id.to_string()])
                        .status() {
                        eprintln!("Failed to kill app process: {}", e);
                    }
                    
                    // Wait for the process to terminate
                    std::thread::sleep(Duration::from_millis(500));
                    
                    // Rebuild the library
                    println!("Rebuilding UI library...");
                    let rebuild_status = Command::new("cargo")
                        .args(["build", "--package", "app"])
                        .status()?;
                        
                    if !rebuild_status.success() {
                        eprintln!("Rebuild failed, waiting for the next change...");
                        continue;
                    }
                    
                    // Start a new instance of the app
                    println!("Restarting application...");
                    return start_app_and_watch();
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