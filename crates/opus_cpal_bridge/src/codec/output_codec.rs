use cpal::SAMPLE_RATE_48K;
use opus::Decoder;

#[derive(Debug)]
pub struct OutputCodec {
    decoder: Decoder,
}

impl OutputCodec {
    pub fn new() -> Result<Self, opus::Error> {
        let decoder = opus::Decoder::new(SAMPLE_RATE_48K, opus::Channels::Mono)?;

        Ok(Self { decoder })
    }

    /// Decode an Opus packet with floating point output.
    ///
    /// To represent packet loss, pass an empty slice &[].
    ///
    /// The return value is the number of samples per channel decoded from the packet.
    pub fn decode(&mut self, packet: &[u8], samples: &mut [f32]) {
        self.decoder
            .decode_float(packet, samples, true)
            .expect("Failed to decode packet");
    }
}
