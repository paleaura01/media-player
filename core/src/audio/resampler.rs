// core/src/audio/resampler.rs

/// Resamples audio data from one sample rate to another
/// 
/// # Arguments
/// * `input` - The input audio samples
/// * `input_rate` - The sample rate of the input in Hz
/// * `output_rate` - The desired output sample rate in Hz
/// * `channels` - The number of audio channels
pub fn resample(input: &[f32], input_rate: u32, output_rate: u32, channels: usize) -> Vec<f32> {
    // Safety checks for invalid inputs
    if input.is_empty() || channels == 0 {
        return Vec::new();
    }
    
    // If rates match, just return a copy of the input
    if input_rate == output_rate {
        return input.to_vec();
    }
    
    // Ensure rates are valid to prevent division by zero
    if input_rate == 0 || output_rate == 0 {
        return input.to_vec();
    }
    
    let ratio = input_rate as f64 / output_rate as f64;
    let input_frames = input.len() / channels;
    
    // Safety check for empty input or zero ratio
    if input_frames == 0 || ratio <= 0.0 {
        return Vec::new();
    }
    
    let output_frames = (input_frames as f64 / ratio) as usize;
    
    // Pre-allocate with exact size for memory safety
    let mut output = Vec::with_capacity(output_frames * channels);
    
    for frame in 0..output_frames {
        let src_frame = frame as f64 * ratio;
        let src_frame_i = src_frame as usize;
        let fract = src_frame - src_frame_i as f64;
        
        // Guard against exceeding input bounds with an extra safety margin
        if src_frame_i >= input_frames.saturating_sub(1) {
            break;
        }
        
        for ch in 0..channels {
            // Extra bounds checking before indexing
            let curr_idx = src_frame_i * channels + ch;
            let next_idx = (src_frame_i + 1) * channels + ch;
            
            if curr_idx >= input.len() || next_idx >= input.len() {
                continue;
            }
            
            let curr = input[curr_idx];
            let next = input[next_idx];
            let sample = curr + fract as f32 * (next - curr);
            output.push(sample);
        }
    }
    
    output
}