// core/src/audio/position.rs
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Tracks playback position and provides timing information
pub struct PlaybackPosition {
    pub total_samples: u64,
    pub current_sample: Arc<AtomicUsize>,
    pub sample_rate: u32,
    // New fields for explicit seek control
    pub seek_requested: Option<Arc<AtomicBool>>,
    pub seek_target: Option<Arc<Mutex<f32>>>,
}

impl PlaybackPosition {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            total_samples: 0,
            current_sample: Arc::new(AtomicUsize::new(0)),
            sample_rate,
            // Initialize the seek control fields
            seek_requested: Some(Arc::new(AtomicBool::new(false))),
            seek_target: Some(Arc::new(Mutex::new(0.0))),
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
            println!("██ DEBUG: Cannot seek - total_samples is 0");
            return;
        }
        
        // Clamp progress to valid range
        let progress = progress.max(0.0).min(1.0);
        
        // Calculate the sample position from progress
        let new_position = (progress as f64 * self.total_samples as f64) as usize;
        
        // Log the seek operation
        println!("██ DEBUG: Position tracker seeking to sample: {} of {} (progress: {:.4})",
            new_position, self.total_samples, progress);
        
        // Update the atomic position
        self.current_sample.store(new_position, Ordering::Relaxed);
        
        println!("██ DEBUG: Position tracker updated current_sample to {}", new_position);
        
        // Also trigger explicit seek request if the functionality exists
        if let Some(req) = &self.seek_requested {
            req.store(true, Ordering::SeqCst);
            
            // Set target position
            if let Some(target) = &self.seek_target {
                if let Ok(mut target_lock) = target.lock() {
                    *target_lock = progress;
                    println!("██ DEBUG: Set seek target to {:.4}", progress);
                }
            }
            
            println!("██ DEBUG: Set seek_requested flag to true");
        }
    }
    
    // Helper method to request a seek operation
    pub fn request_seek(&self, progress: f32) {
        // Clamp progress to valid range
        let progress = progress.max(0.0).min(1.0);
        
        println!("██ DEBUG: PlaybackPosition.request_seek({:.4}) called", progress);
        
        // Set the target position
        if let Some(target) = &self.seek_target {
            if let Ok(mut target_lock) = target.lock() {
                *target_lock = progress;
                println!("██ DEBUG: Set seek target to {:.4}", progress);
            } else {
                println!("██ DEBUG: Failed to lock seek target");
            }
        } else {
            println!("██ DEBUG: No seek target available");
        }
        
        // Set the request flag
        if let Some(req) = &self.seek_requested {
            req.store(true, Ordering::SeqCst);
            println!("██ DEBUG: Set seek_requested flag to true");
        } else {
            println!("██ DEBUG: No seek request flag available");
        }
    }
}