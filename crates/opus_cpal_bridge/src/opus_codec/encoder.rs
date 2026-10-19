use opus::Bitrate;

use crate::opus_codec::{
    OPUS_DEFAULT_BITRATE, OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE, frame_duration::FrameSize,
};

#[derive(Debug)]
pub struct Encoder {
    #[allow(clippy::struct_field_names)]
    encoder: opus::Encoder,
    sample_rate: u32,
    bitrate: u32,
}

impl Encoder {
    pub fn new(sample_rate: u32, frame_duration: FrameSize) -> Result<Self, opus::Error> {
        let mut encoder =
            opus::Encoder::new(sample_rate, opus::Channels::Mono, opus::Application::Voip)?;
        encoder.set_bitrate(Bitrate::Bits(OPUS_DEFAULT_BITRATE as i32))?;
        // Variable bitrate must be `false`.
        // VBR leaks encrypted speech through packet sizes.
        encoder.set_vbr(false)?;
        encoder.set_inband_fec(true)?;
        encoder.set_packet_loss_perc(i32::from(OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE))?;

        Ok(Self {
            encoder,
            sample_rate,
            bitrate: OPUS_DEFAULT_BITRATE,
        })
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
        self.encoder.set_packet_loss_perc(i32::from(value))
    }

    /// Set the encoder's bitrate.
    #[inline]
    pub fn set_bitrate(&mut self, bitrate: u32) -> Result<(), opus::Error> {
        self.encoder.set_bitrate(Bitrate::Bits(bitrate as i32))?;
        // Min/max come from `OPUS_SET_BITRATE_REQUEST` in
        // https://gitlab.xiph.org/xiph/opus/-/raw/1632152b83b8ab4e28393bca94450796b71b0201/src/opus_encoder.c
        self.bitrate = bitrate.clamp(500, 300_000);
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn get_bitrate(&self) -> u32 {
        self.bitrate
    }
}
