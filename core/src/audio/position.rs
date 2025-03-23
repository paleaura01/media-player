// core/src/audio/position.rs
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Tracks playback position and provides timing information
pub struct PlaybackPosition {
    pub total_samples: u64,
    pub current_sample: Arc<AtomicUsize>,
    pub sample_rate: u32,
}

impl PlaybackPosition {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            total_samples: 0,
            current_sample: Arc::new(AtomicUsize::new(0)),
            sample_rate,
        }
    }

    pub fn set_total_samples(&mut self, total_samples: u64) {
        self.total_samples = total_samples;
    }

    pub fn update_current_sample(&self, samples: usize) {
        let current = self.current_sample.load(Ordering::Relaxed);
        let new_value = current.saturating_add(samples);
        self.current_sample.store(new_value, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.current_sample.store(0, Ordering::Relaxed);
    }

    pub fn progress(&self) -> f32 {
        if self.total_samples == 0 {
            return 0.0;
        }
        let current = self.current_sample.load(Ordering::Relaxed) as f64;
        let total = self.total_samples as f64;
        (current / total).min(1.0) as f32
    }

    pub fn position(&self) -> Duration {
        let current = self.current_sample.load(Ordering::Relaxed);
        // Prevent division by zero
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = current as f64 / (sample_rate * 2.0); // Stereo
        Duration::from_secs_f64(seconds)
    }

    pub fn duration(&self) -> Duration {
        // Prevent division by zero
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = self.total_samples as f64 / (sample_rate * 2.0); // Stereo
        Duration::from_secs_f64(seconds)
    }

    pub fn seek(&self, progress: f32) {
        if self.total_samples == 0 {
            return;
        }
        
        // Clamp progress to valid range
        let progress = progress.max(0.0).min(1.0);
        
        // Calculate the sample position from progress
        let new_position = (progress as f64 * self.total_samples as f64) as usize;
        
        // Log the seek operation
        println!("Position tracker seeking to sample: {} of {} (progress: {:.4})",
            new_position, self.total_samples, progress);
        
        // Update the atomic position
        self.current_sample.store(new_position, Ordering::Relaxed);
    }
}