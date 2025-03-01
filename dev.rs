use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting development environment...");
    
    // Ensure directories exist
    for dir in &["./ui", "./assets", "./data"] {
        if !Path::new(dir).exists() {
            std::fs::create_dir_all(dir)?;
            println!("Created directory: {}", dir);
        }
    }
    
    // Create an empty playlists.bin if it doesn't exist
    let playlists_path = Path::new("./data/playlists.bin");
    if !playlists_path.exists() {
        // Create an empty file
        std::fs::write(playlists_path, &[])?;
        println!("Created empty playlists.bin");
    }
    
    // Build UI components initially
    println!("Building UI library...");
    let status = Command::new("cargo")
        .args(["build", "--package", "ui"])
        .status()?;
    
    if !status.success() {
        eprintln!("Failed to build UI library!");
        return Err("Build failed".into());
    }
    
    // Start the main application
    println!("Starting main application...");
    let mut app_process = Command::new("cargo")
        .args(["run", "--package", "app"])
        .spawn()?;
    
    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx)?;
    
    // Watch the ui directory
    watcher.watch(Path::new("./ui/src"), RecursiveMode::Recursive)?;
    println!("Watching for changes in ui/src...");
    println!("Press Ctrl+C to stop");
    
    // Main loop
    let mut is_rebuilding = false;
    
    ctrlc::set_handler(move || {
        println!("\nShutting down development environment...");
        std::process::exit(0);
    })?;
    
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if is_file_change(&event) && !is_rebuilding {
                    is_rebuilding = true;
                    println!("Change detected, rebuilding UI...");
                    
                    let status = Command::new("cargo")
                        .args(["build", "--package", "ui"])
                        .status()?;
                    
                    if status.success() {
                        println!("UI rebuilt successfully!");
                    } else {
                        eprintln!("Failed to rebuild UI!");
                    }
                    
                    is_rebuilding = false;
                }
            }
            Err(e) if e.is_timeout() => {
                // Just a timeout, continue
            }
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
            }
            Ok(None) => {
                // App is still running
            }
            Err(e) => {
                eprintln!("Error checking app status: {}", e);
                break;
            }
        }
    }
    
    // Clean up
    if let Err(e) = app_process.kill() {
        eprintln!("Failed to kill app process: {}", e);
    }
    
    Ok(())
}

fn is_file_change(event: &Event) -> bool {
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            event.paths.iter().any(|p| {
                p.extension().map_or(false, |ext| {
                    ext == "rs" || ext == "toml"
                })
            })
        }
        _ => false,
    }
}