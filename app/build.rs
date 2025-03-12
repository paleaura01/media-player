use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;

fn main() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    // In debug mode, use a constant build ID so that the generated file does not change every time.
    let build_id = if profile == "release" {
         SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    } else {
         0
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = format!("{}/build_id.rs", out_dir);

    let new_content = format!("pub const BUILD_ID: u64 = {};\n", build_id);

    // Only rewrite the file if its content has changed to prevent forcing full rebuilds
    let need_write = if let Ok(existing) = fs::read_to_string(&dest_path) {
         existing != new_content
    } else {
         true
    };

    if need_write {
         let mut f = File::create(&dest_path).unwrap();
         f.write_all(new_content.as_bytes()).unwrap();
    }

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
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_id);
}
