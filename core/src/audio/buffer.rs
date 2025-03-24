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
    /// Create a new ring buffer with the given capacity.
    /// The actual capacity will be at least 1024 to avoid extremely small buffers.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1024);
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            samples_available: 0,
        }
    }

    /// Return the capacity of this ring buffer.
    /// (Added so external code can safely read the capacity, instead of
    /// accessing the private `capacity` field.)
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Write as many samples as possible into the buffer.
    /// Returns the number of samples actually written.
    pub fn write(&mut self, samples: &[f32]) -> usize {
        if samples.is_empty() || self.capacity == 0 {
            return 0;
        }

        let mut written = 0;
        for &sample in samples {
            if self.samples_available >= self.capacity {
                break;
            }

            if self.write_pos < self.buffer.len() {
                self.buffer[self.write_pos] = sample;
                self.write_pos = (self.write_pos + 1) % self.capacity;
                self.samples_available += 1;
                written += 1;
            }
        }

        written
    }

    /// Read up to `output.len()` samples from the buffer, returning how many samples were read.
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        if output.is_empty() || self.samples_available == 0 {
            return 0;
        }

        let to_read = output.len().min(self.samples_available);

        for i in 0..to_read {
            if i < output.len() && self.read_pos < self.buffer.len() {
                output[i] = self.buffer[self.read_pos];
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }

        self.samples_available = self.samples_available.saturating_sub(to_read);
        to_read
    }

    /// Return the number of samples currently available to read from the buffer.
    pub fn available(&self) -> usize {
        self.samples_available
    }
}
