use std::env;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;

fn main() {
    // Get current timestamp as build ID
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Write the timestamp to a generated file
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = format!("{}/build_id.rs", out_dir);
    let mut f = File::create(&dest_path).unwrap();
    
    // Include the timestamp in the build
    writeln!(f, "pub const BUILD_ID: u64 = {};", timestamp).unwrap();
    
    // Force recompilation when any source file changes
    println!("cargo:rerun-if-changed=src");
    
    // Also specifically watch UI files
    if Path::new("src/ui").exists() {
        println!("cargo:rerun-if-changed=src/ui");
        println!("cargo:rerun-if-changed=src/ui/mod.rs");
        println!("cargo:rerun-if-changed=src/ui/styles.rs");
        println!("cargo:rerun-if-changed=src/ui/library_view.rs");
        println!("cargo:rerun-if-changed=src/ui/player_view.rs");
        println!("cargo:rerun-if-changed=src/ui/playlist_view.rs");
    }
    
    // Export the build timestamp as an environment variable
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
}