// core/src/audio/mod.rs
pub mod buffer;
pub mod decoder;
pub mod device;
pub mod position;
pub mod resampler;
pub mod codec_registry;  // Add this line

// Re-export key types
pub use buffer::AudioRingBuffer;
pub use position::PlaybackPosition;
pub use codec_registry::register_all_codecs;  // Add this line