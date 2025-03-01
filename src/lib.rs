// src/lib.rs
pub mod audio;
pub mod player;

// Ensure Player and PlayerState are publicly accessible
pub use crate::player::{Player, PlayerState};