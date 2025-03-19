// core/src/audio/buffer.rs
/// Ring buffer for audio samples to allow smoother playback
pub struct AudioRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    samples_available: usize,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        // Ensure minimum capacity to prevent issues with tiny buffers
        let capacity = capacity.max(1024);
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            samples_available: 0,
        }
    }
    
    pub fn write(&mut self, samples: &[f32]) -> usize {
        // Safety check for empty input
        if samples.is_empty() || self.capacity == 0 {
            return 0;
        }
        
        let mut written = 0;
        for &sample in samples {
            if self.samples_available >= self.capacity {
                break;
            }
            
            // Double-check bounds for extra safety
            if self.write_pos < self.buffer.len() {
                self.buffer[self.write_pos] = sample;
                self.write_pos = (self.write_pos + 1) % self.capacity;
                self.samples_available += 1;
                written += 1;
            }
        }
        
        written
    }
    
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        // Safety check for empty output buffer
        if output.is_empty() || self.samples_available == 0 {
            return 0;
        }
        
        let to_read = output.len().min(self.samples_available);
        
        for i in 0..to_read {
            // Double-check bounds for extra safety
            if i < output.len() && self.read_pos < self.buffer.len() {
                output[i] = self.buffer[self.read_pos];
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
        
        // Update samples_available safely using saturating_sub to prevent underflow
        self.samples_available = self.samples_available.saturating_sub(to_read);
        to_read
    }
    
    pub fn available(&self) -> usize {
        self.samples_available
    }
}