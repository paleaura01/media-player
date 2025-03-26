use audiopus::{Channels, Decoder};
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::codecs::{CodecDescriptor, CodecParameters, Decoder as SymphoniaDecoder, DecoderOptions, CODEC_TYPE_OPUS};
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::Packet;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use log::{debug, error, info, warn};

pub struct OpusDecoder {
    decoder: Decoder,
    params: CodecParameters,
    sample_rate: u32,
    channels: usize,
    frame_size_ms: usize, // Frame size in milliseconds (2.5, 5, 10, 20, 40, or 60)
}

impl OpusDecoder {
    pub fn try_new(params: &CodecParameters, _options: &DecoderOptions) -> Result<Self> {
        // Opus always uses 48kHz internally
        let sample_rate = params.sample_rate.unwrap_or(48000);
        let channels = params.channels.map(|c| c.count()).unwrap_or(2);
        
        // Convert symphonia channels to audiopus channels
        let opus_channels = match channels {
            1 => Channels::Mono,
            2 => Channels::Stereo,
            _ => return Err(Error::Unsupported("Only mono and stereo are supported in Opus".into())),
        };
        
        // Create the Opus decoder
        let decoder = Decoder::new(sample_rate, opus_channels)
            .map_err(|e| Error::Unsupported(format!("Failed to create Opus decoder: {}", e)))?;
            
        // Default to 20ms frames
        let frame_size_ms = 20; 
        
        info!("Created Opus decoder: {} Hz, {} channels", sample_rate, channels);
        
        Ok(Self {
            decoder,
            params: params.clone(),
            sample_rate,
            channels,
            frame_size_ms,
        })
    }
    
    // Helper to calculate frame size based on sample rate
    fn frame_size(&self) -> usize {
        // Opus uses frame sizes that are powers of 2 in samples per channel
        (self.sample_rate as usize * self.frame_size_ms) / 1000
    }
}

impl SymphoniaDecoder for OpusDecoder {
    fn try_new(params: &CodecParameters, options: &DecoderOptions) -> Result<Self> {
        Self::try_new(params, options)
    }

    fn reset(&mut self) -> Result<()> {
        // Reset the decoder state
        self.decoder = Decoder::new(self.sample_rate, if self.channels == 1 { Channels::Mono } else { Channels::Stereo })
            .map_err(|e| Error::Unsupported(format!("Failed to reset Opus decoder: {}", e)))?;
        Ok(())
    }

    fn codec_params(&self) -> &CodecParameters {
        &self.params
    }

    fn decode(&mut self, packet: &Packet) -> Result<AudioBuffer<f32>> {
        // Decode the opus packet
        let frame_size = self.frame_size();
        let mut output = vec![0.0; frame_size * self.channels];
        
        self.decoder.decode_float(packet.data(), &mut output, false)
            .map_err(|e| Error::DecodeError(format!("Opus decode error: {}", e)))?;
            
        // Create an audio buffer from the decoded data
        let spec = self.params.spec()?;
        let duration = frame_size as u64;
        
        let mut buffer = AudioBuffer::new(duration, spec);
        
        // Copy decoded samples to the audio buffer
        if self.channels == 1 {
            buffer.copy_interleaved_ref(&output);
        } else {
            // For stereo, deinterleave
            let planes = buffer.planes_mut();
            if planes.len() >= 2 {
                for (i, sample) in output.iter().enumerate() {
                    let channel = i % self.channels;
                    let frame = i / self.channels;
                    
                    if frame < planes[0].len() {
                        planes[channel][frame] = *sample;
                    }
                }
            }
        }
        
        Ok(buffer)
    }

    fn last_decoded(&self) -> Option<AudioBuffer<f32>> {
        None
    }

    fn finalize(&mut self) -> Result<Option<AudioBuffer<f32>>> {
        Ok(None)
    }
}

pub struct OpusDecoderFactory;

impl symphonia::core::codecs::DecoderFactory for OpusDecoderFactory {
    fn new(&self) -> Box<dyn symphonia::core::codecs::Decoder> {
        // Create a dummy codec parameters object
        let mut params = CodecParameters::new();
        params.codec = CODEC_TYPE_OPUS;
        params.sample_rate = Some(48000);
        params.channels = Some(symphonia::core::audio::Channels::FRONT_LEFT | symphonia::core::audio::Channels::FRONT_RIGHT);
        
        // Create a new decoder with default options
        let options = DecoderOptions::default();
        
        match OpusDecoder::try_new(&params, &options) {
            Ok(decoder) => Box::new(decoder),
            Err(err) => {
                error!("Failed to create Opus decoder: {}", err);
                // Return a placeholder decoder that will fail when used
                Box::new(PlaceholderDecoder::new())
            }
        }
    }

    fn try_new(&self, params: &CodecParameters, options: &DecoderOptions) -> Result<Box<dyn symphonia::core::codecs::Decoder>> {
        let decoder = OpusDecoder::try_new(params, options)?;
        Ok(Box::new(decoder))
    }

    fn get_codec_type(&self) -> u32 {
        CODEC_TYPE_OPUS
    }
}

// Simple placeholder decoder that always returns errors
struct PlaceholderDecoder;

impl PlaceholderDecoder {
    fn new() -> Self {
        Self
    }
}

impl SymphoniaDecoder for PlaceholderDecoder {
    fn try_new(_: &CodecParameters, _: &DecoderOptions) -> Result<Self> {
        Ok(Self)
    }

    fn reset(&mut self) -> Result<()> {
        Err(Error::DecodeError("Placeholder decoder cannot be used".into()))
    }

    fn codec_params(&self) -> &CodecParameters {
        panic!("Placeholder decoder should not be used")
    }

    fn decode(&mut self, _: &Packet) -> Result<AudioBuffer<f32>> {
        Err(Error::DecodeError("Placeholder decoder cannot decode packets".into()))
    }

    fn last_decoded(&self) -> Option<AudioBuffer<f32>> {
        None
    }

    fn finalize(&mut self) -> Result<Option<AudioBuffer<f32>>> {
        Ok(None)
    }
}