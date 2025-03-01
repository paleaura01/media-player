// src/lib.rs
pub mod audio;
pub mod player;
pub mod gui;  // Add this line to expose the GUI module

// Re-export key types
pub use player::{Player, PlayerState};