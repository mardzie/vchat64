use cpal::SAMPLE_RATE_48K;
use opus::{Bitrate, Encoder};

use crate::OPUS_DEFAULT_BITRATE;

#[derive(Debug)]
pub struct InputCodec {
    encoder: Encoder,
}

impl InputCodec {
    pub fn new() -> Result<Self, opus::Error> {
        let mut encoder = opus::Encoder::new(
            SAMPLE_RATE_48K,
            opus::Channels::Mono,
            opus::Application::Voip,
        )?;
        encoder.set_bitrate(Bitrate::Bits(OPUS_DEFAULT_BITRATE))?;
        encoder.set_vbr(true)?;
        encoder.set_inband_fec(true)?;
        encoder.set_packet_loss_perc(10)?;

        Ok(Self { encoder })
    }

    /// Encode an Opus frame from floating point input.
    pub fn encode(&mut self, samples: &[f32], packet: &mut [u8]) {
        let len = self
            .encoder
            .encode_float(samples, packet)
            .expect("Failed to encode to opus frame");
    }
}
