// src/lib.rs
pub mod audio;
pub mod player;
pub mod gui;

// Ensure Player and PlayerState are publicly accessible
pub use crate::player::{Player, PlayerState};