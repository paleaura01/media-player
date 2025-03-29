// core/src/audio/resampler.rs
extern crate ffmpeg_next as ffmpeg;

use anyhow::Result;
use log::debug;

// For simple sample rate conversion
pub fn resample(
    input: &[f32],
    input_rate: u32,
    output_rate: u32, 
    channels: usize
) -> Vec<f32> {
    // For simple cases or no resampling needed
    if input_rate == output_rate {
        return input.to_vec();
    }
    
    // Use more complete function
    match resample_buffer(input, input_rate, channels, output_rate, channels) {
        Ok(output) => output,
        Err(e) => {
            debug!("Resampling failed: {}, returning original", e);
            input.to_vec()
        }
    }
}

// More complete resampling function
pub fn resample_buffer(
    input: &[f32],
    input_rate: u32,
    input_channels: usize,
    output_rate: u32, 
    output_channels: usize
) -> Result<Vec<f32>> {
    // If no resampling needed and channel count matches
    if input_rate == output_rate && input_channels == output_channels {
        return Ok(input.to_vec());
    }
    
    // Check for empty input
    if input.is_empty() || input_channels == 0 || output_channels == 0 {
        return Ok(Vec::new());
    }
    
    // Create a channel layout for input
    let in_layout = match input_channels {
        1 => ffmpeg::channel_layout::ChannelLayout::MONO,
        2 => ffmpeg::channel_layout::ChannelLayout::STEREO,
        _ => ffmpeg::channel_layout::ChannelLayout::default(input_channels as i32),
    };
    
    // Create a channel layout for output
    let out_layout = match output_channels {
        1 => ffmpeg::channel_layout::ChannelLayout::MONO,
        2 => ffmpeg::channel_layout::ChannelLayout::STEREO,
        _ => ffmpeg::channel_layout::ChannelLayout::default(output_channels as i32),
    };
    
    // Create resampler
    let mut resampler = ffmpeg::software::resampling::Context::get(
        ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
        in_layout,
        input_rate,
        ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
        out_layout,
        output_rate,
    )?;
    
    // Calculate input frame count
    let frame_count = input.len() / input_channels;
    
    // Create FFmpeg frame for input
    let mut input_frame = ffmpeg::frame::Audio::new(
        ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
        frame_count,
        in_layout,
    );
    
    // Set input frame parameters
    input_frame.set_rate(input_rate);
    input_frame.set_samples(frame_count);
    
    // Copy data to input frame - directly use the slice, not a Result
    let plane = input_frame.plane_mut::<f32>(0);
    for (i, &sample) in input.iter().enumerate() {
        if i < plane.len() {
            plane[i] = sample;
        }
    }
    
    // Create an empty output frame (resampling will fill it)
    let mut output_frame = ffmpeg::frame::Audio::empty();
    
    // Perform resampling
    resampler.run(&input_frame, &mut output_frame)?;
    
    // Extract data from output frame - directly use the slice, not a Result
    let data = output_frame.plane::<f32>(0);
    let output_samples = output_frame.samples() * output_channels;
    
    // Create the output vector
    let mut output = Vec::with_capacity(output_samples);
    for i in 0..output_samples {
        if i < data.len() {
            output.push(data[i]);
        } else {
            output.push(0.0); // Pad with zeros if needed
        }
    }
    
    debug!("Resampled {}Hz {}ch -> {}Hz {}ch", input_rate, input_channels, output_rate, output_channels);
    
    Ok(output)
}