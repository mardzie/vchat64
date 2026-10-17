use opus::Bitrate;

use crate::{
    SAMPLE_RATE_48K,
    codec::{OPUS_DEFAULT_BITRATE, OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE},
};

#[derive(Debug)]
pub struct Encoder {
    encoder: opus::Encoder,
}

impl Encoder {
    pub fn new() -> Result<Self, opus::Error> {
        let mut encoder = opus::Encoder::new(
            SAMPLE_RATE_48K,
            opus::Channels::Mono,
            opus::Application::Voip,
        )?;
        encoder.set_bitrate(Bitrate::Bits(OPUS_DEFAULT_BITRATE))?;
        // Variable bitrate must be `false`.
        // VBR leaks encrypted speech through packet sizes.
        encoder.set_vbr(false)?;
        encoder.set_inband_fec(true)?;
        encoder.set_packet_loss_perc(OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE)?;

        Ok(Self { encoder })
    }

    /// Encode an Opus frame from floating point input.
    #[inline]
    pub fn encode(&mut self, samples: &[f32], packet: &mut [u8]) -> Result<usize, opus::Error> {
        self.encoder.encode_float(samples, packet)
    }

    /// Sets the encoder’s expected packet loss percentage.
    #[inline]
    pub fn set_packet_loss_percentage(&mut self, value: u8) -> Result<(), opus::Error> {
        debug_assert!(value <= 100);
        self.encoder.set_packet_loss_perc(value as i32)
    }
}
