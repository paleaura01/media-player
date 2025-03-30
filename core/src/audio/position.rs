// core/src/audio/position.rs
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{debug, info};

pub struct PlaybackPosition {
    pub total_samples: u64,
    pub current_sample: Arc<AtomicUsize>,
    pub sample_rate: u32,
    pub channel_count: usize,
    pub seek_requested: Option<Arc<AtomicBool>>,
    pub seek_target: Option<Arc<Mutex<f32>>>,
    pub buffer_health: Option<f32>,
    pub clear_buffers: bool,
    // Use a proper mutex for log timestamps
    last_progress_log: Arc<Mutex<Instant>>,
}

impl PlaybackPosition {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            total_samples: 0,
            current_sample: Arc::new(AtomicUsize::new(0)),
            sample_rate,
            channel_count: 2,
            seek_requested: Some(Arc::new(AtomicBool::new(false))),
            seek_target: Some(Arc::new(Mutex::new(0.0))),
            buffer_health: None,
            clear_buffers: false,
            last_progress_log: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    pub fn update_buffer_health(&mut self, available: usize, capacity: usize) {
        if capacity > 0 {
            self.buffer_health = Some(available as f32 / capacity as f32);
        } else {
            self.buffer_health = None;
        }
    }

    pub fn set_total_samples(&mut self, total_samples: u64) {
        self.total_samples = total_samples;
        info!("Set total_samples to {} (channel_count = {})", 
              total_samples, self.channel_count);
    }

    pub fn set_channel_count(&mut self, channels: usize) {
        self.channel_count = channels;
        info!("Set channel_count to {}", channels);
    }

    pub fn update_current_sample(&self, samples: usize) {
        let current = self.current_sample.load(Ordering::Relaxed);
        let sample_frames = samples / self.channel_count;
        let new_value = current.saturating_add(sample_frames);
        self.current_sample.store(new_value, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.current_sample.store(0, Ordering::Relaxed);
        debug!("Reset current_sample to 0");
    }

    pub fn progress(&self) -> f32 {
        if self.total_samples == 0 {
            return 0.0;
        }
        
        let current_frame = self.current_sample.load(Ordering::Relaxed) as f64;
        let total_frames = (self.total_samples as f64) / (self.channel_count as f64);
        let progress = (current_frame / total_frames).min(1.0) as f32;
        
        // Only log progress updates once per second using proper mutex
        let should_log = {
            if let Ok(mut last_log) = self.last_progress_log.lock() {
                if last_log.elapsed() > Duration::from_secs(1) {
                    *last_log = Instant::now();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        if should_log {
            debug!("Calculating progress: current_frame={}, total_frames={}, progress={:.4}", 
                  current_frame, total_frames, progress);
        }
                  
        progress
    }

    pub fn position(&self) -> Duration {
        let current_frame = self.current_sample.load(Ordering::Relaxed);
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = current_frame as f64 / sample_rate;
        Duration::from_secs_f64(seconds)
    }

    pub fn duration(&self) -> Duration {
        let sample_rate = self.sample_rate.max(1) as f64;
        let total_frames = (self.total_samples as f64) / (self.channel_count as f64);
        let seconds = total_frames / sample_rate;
        Duration::from_secs_f64(seconds)
    }

    pub fn seek(&self, progress: f32) {
        if self.total_samples == 0 {
            debug!("Cannot seek - total_samples is 0");
            return;
        }
        let prog = progress.clamp(0.0, 1.0);
        
        let total_frames = self.total_samples / self.channel_count as u64;
        let new_frame_position = (prog as f64 * total_frames as f64) as usize;
        
        debug!("Direct seek: progress={:.4}, total_frames={}, new_frame_position={}", 
              prog, total_frames, new_frame_position);
                  
        self.current_sample.store(new_frame_position, Ordering::Relaxed);
    }

    pub fn request_seek(&mut self, fraction: f32) {
        let frac = fraction.clamp(0.0, 1.0);
        
        info!("Seek requested to {:.4} ({:.2}%)", frac, frac * 100.0);
        
        let current_frame = self.current_sample.load(Ordering::Relaxed);
        let total_frames = self.total_samples / self.channel_count as u64;
        let target_frame = (frac * total_frames as f32) as usize;
        
        info!("Current frame: {}/{} ({:.2}%), Target frame: {}/{} ({:.2}%)",
             current_frame, total_frames, 
             (current_frame as f64 / total_frames as f64) * 100.0,
             target_frame, total_frames, frac * 100.0);
        
        if let Some(flag) = &self.seek_requested {
            let previous = flag.swap(true, Ordering::SeqCst);
            if previous {
                debug!("Note: Overwriting a previous seek request that was not yet processed");
            }
        } else {
            debug!("seek_requested flag is not initialized");
        }
        
        if let Some(target) = &self.seek_target {
            if let Ok(mut tgt_lock) = target.lock() {
                *tgt_lock = frac;
                debug!("Set seek target to {:.4}", frac);
            } else {
                debug!("Failed to acquire lock for seek target");
            }
        } else {
            debug!("seek_target is not initialized");
        }
    }

    pub fn set_current_frame(&self, frame_index: usize) {
        self.current_sample.store(frame_index, Ordering::SeqCst);
        debug!("Set current_frame to {}", frame_index);
    }
}