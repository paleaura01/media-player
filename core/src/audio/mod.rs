// core/src/audio/mod.rs
pub mod buffer;
pub mod decoder;
pub mod device;
pub mod position;
pub mod resampler;

// Re-export key types
pub use buffer::AudioRingBuffer;
pub use position::PlaybackPosition;
pub use decoder::{initialize_ffmpeg, get_supported_extensions, is_supported_audio_format};