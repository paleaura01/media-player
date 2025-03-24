use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = current as f64 / (sample_rate * 2.0); // stereo assumption
        Duration::from_secs_f64(seconds)
    }

    pub fn duration(&self) -> Duration {
        let sample_rate = self.sample_rate.max(1) as f64;
        let seconds = self.total_samples as f64 / (sample_rate * 2.0);
        Duration::from_secs_f64(seconds)
    }

    pub fn seek(&self, progress: f32) {
        if self.total_samples == 0 {
            log::debug!("Cannot seek - total_samples is 0");
            return;
        }
        let prog = progress.clamp(0.0, 1.0);
        let new_position = (prog as f64 * self.total_samples as f64) as usize;
        self.current_sample.store(new_position, Ordering::Relaxed);
    }

    pub fn request_seek(&self, fraction: f32) {
        let frac = fraction.clamp(0.0, 1.0);
        if let Some(flag) = &self.seek_requested {
            flag.store(true, Ordering::SeqCst);
        }
        if let Some(target) = &self.seek_target {
            if let Ok(mut tgt_lock) = target.lock() {
                *tgt_lock = frac;
            }
        }
        log::debug!("request_seek called -> fraction: {:.4}", frac);
    }

    // (Optional) If we want a dedicated setter:
    pub fn set_current_sample(&self, sample_index: usize) {
        self.current_sample.store(sample_index, Ordering::Relaxed);
    }
}
