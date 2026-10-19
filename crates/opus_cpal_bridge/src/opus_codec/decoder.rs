use crate::opus_codec::{frame_size::FrameSize, opus_slice::OpusSlice};

#[derive(Debug)]
pub struct Decoder {
    decoder: opus::Decoder,
    sample_rate: u32,
}

impl Decoder {
    pub fn new(sample_rate: u32, frame_duration: FrameSize) -> Result<Self, opus::Error> {
        Ok(Self {
            decoder: opus::Decoder::new(sample_rate, opus::Channels::Mono)?,
            sample_rate,
        })
    }

    /// Decode an Opus packet with floating point output.
    ///
    /// To represent packet loss, pass `DecodeInput::Plc`.
    ///
    /// The return value is the number of samples per channel decoded from the packet.
    #[inline]
    pub fn decode(&mut self, input: DecodeInput, output: &mut [f32]) -> Result<usize, opus::Error> {
        let (packet, fec): (&[u8], bool) = match input {
            DecodeInput::Packet(slice) => (slice.get(), false),
            DecodeInput::Fec { next } => (next.get(), true),
            DecodeInput::Plc => (&[] as &[u8], false),
        };

        self.decoder.decode_float(packet, output, fec)
    }

    pub fn set_gain(&mut self, gain: i32) -> Result<(), opus::Error> {
        self.decoder.set_gain(gain)
    }
}

/// Input for one playout slot.
#[derive(Debug)]
pub enum DecodeInput<'a> {
    /// The slot's own packet arrived.
    ///
    /// -> `decode_float(pkt, &mut out[..FRAME], false)`
    Packet(OpusSlice<'a>),

    /// The slot is lost but the next packet arrived: decode the copy embedded in it.
    /// (Don't forget to call again with `DecodeInput::Packet(next)` or else next isn't being decoded.)
    ///
    /// -> `decode_float(next, &mut out[..FRAME], true)`
    ///
    /// (FEC = Forward Error Correction)
    Fec { next: OpusSlice<'a> },

    /// The slot and the next packet are both missing.
    ///
    /// -> `decode_float(&[], &mut out[..FRAME], false)`
    ///
    /// (PLC = Packet Loss Concealment)
    Plc,
}
