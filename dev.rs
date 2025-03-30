use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use ctrlc;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct WindowPosition {
    x: Option<i32>,
    y: Option<i32>,
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
    
    // Build the project initially
    println!("Building project...");
    let status = Command::new("cargo")
        .args(["build", "--bin", "media-player-app", "--package", "app"])
        .status()?;
    
    if !status.success() {
        eprintln!("Failed to build application!");
        return Err("Build failed".into());
    }
    
    // Start the application
    println!("Starting main application...");
    
    let mut args = Vec::<String>::new();
    args.push("run".to_string());
    args.push("--bin".to_string());
    args.push("media-player-app".to_string());
    args.push("--package".to_string());
    args.push("app".to_string());
    
    let mut app_process = Command::new("cargo")
        .args(&args)
        .spawn()?;
    
        let _app_process_id = app_process.id();

    
    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = recommended_watcher(tx)?;
    
    watcher.watch(Path::new("./app/src"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("./core/src"), RecursiveMode::Recursive)?; // Also watch core library
    println!("Watching for changes in app/src and core/src...");
    println!("Press Ctrl+C to stop");
    
    // Main loop with improved Ctrl+C handler
    let app_process_id_for_handler = app_process.id();
    ctrlc::set_handler(move || {
        println!("\nShutting down development environment...");
        
        // Properly terminate the app process first with a more reliable approach
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &app_process_id_for_handler.to_string()])
                .status() {
                eprintln!("Failed to kill app process: {}", e);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            if let Err(e) = Command::new("kill")
                .args(["-9", &app_process_id_for_handler.to_string()])
                .status() {
                eprintln!("Failed to kill app process: {}", e);
            }
        }
        
        std::thread::sleep(Duration::from_millis(1000));
        std::process::exit(0);
    })?;
    
    let mut last_rebuild_time = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(750);
    
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if is_file_change(&event) && last_rebuild_time.elapsed() > debounce_duration {
                    last_rebuild_time = std::time::Instant::now();
                    
                    println!("\n\n===== CHANGE DETECTED =====");
                    for path in &event.paths {
                        if let Some(path_str) = path.to_str() {
                            println!("Changed file: {}", path_str);
                        }
                    }
                    
                    // Wait for FS to settle
                    std::thread::sleep(Duration::from_millis(500));
                    
                    println!("Stopping application for rebuild...");
                    #[cfg(target_os = "windows")]
                    {
                        if let Err(e) = Command::new("taskkill")
                            .args(["/F", "/T", "/PID", &app_process.id().to_string()])
                            .status() {
                            eprintln!("Failed to kill app process: {}", e);
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        if let Err(e) = Command::new("kill")
                            .args(["-9", &app_process.id().to_string()])
                            .status() {
                            eprintln!("Failed to kill app process: {}", e);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(500));
                    
                    println!("Rebuilding application...");
                    let rebuild_success = {
                        let status = Command::new("cargo")
                            .args(["build", "--bin", "media-player-app", "--package", "app"])
                            .status()?;
                        status.success()
                    };
                    
                    if rebuild_success {
                        println!("Build successful, restarting application...");
                        match Command::new("cargo")
                            .args(&args)
                            .spawn() {
                            Ok(process) => {
                                app_process = process;
                                println!("Application restarted successfully.");
                            },
                            Err(e) => {
                                eprintln!("Failed to restart application: {}", e);
                            }
                        }
                    } else {
                        eprintln!("Build failed, will not restart application.");
                    }
                }
            },
            Ok(Err(e)) => {
                eprintln!("Error receiving file event: {:?}", e);
            },
            Err(RecvTimeoutError::Timeout) => { },
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
                break;
            }
        }
        
        match app_process.try_wait() {
            Ok(Some(status)) => {
                println!("Application exited with status: {}", status);
                break;
            },
            Ok(None) => { },
            Err(e) => {
                eprintln!("Error checking app status: {}", e);
                break;
            }
        }
    }
    
    println!("Attempting to clean up application process.");
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &app_process.id().to_string()])
            .status() {
            eprintln!("Failed to kill app process: {}", e);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Err(e) = Command::new("kill")
            .args(["-9", &app_process.id().to_string()])
            .status() {
            eprintln!("Failed to kill app process: {}", e);
        }
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