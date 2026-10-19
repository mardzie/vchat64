/// Legal Opus frame durations (libopus >= 1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameSize {
    /// CELT-only => no in-band FEC
    Ms2_5 = 1,
    /// CELT-only => no in-band FEC
    Ms5 = 2,
    Ms10 = 4,
    Ms20 = 8,
    Ms40 = 16,
    Ms60 = 24,
    Ms80 = 32,
    Ms100 = 40,
    Ms120 = 48,
}

impl FrameSize {
    /// 2.5 ms in 1000 ms (1000 / 2.5)
    const MIN_FRAME_SIZE_IN_SECOND: u32 = (1000.0 / 2.5) as u32;

    #[must_use]
    pub const fn samples(self, sample_rate: u32) -> usize {
        (sample_rate / Self::MIN_FRAME_SIZE_IN_SECOND) as usize * self as usize
    }

    #[must_use]
    pub const fn cbr_packet_len(self, sample_rate: u32, bitrate: u32) -> usize {
        let samples = self.samples(sample_rate) as i64;
        Self::div_round(
            bitrate as i64 * samples,
            sample_rate as i64 * u8::BITS as i64,
        ) as usize
    }

    /// `a / b` rounded to nearest, ties up. Requires `b > 0`
    #[must_use]
    const fn div_round(a: i64, b: i64) -> i64 {
        debug_assert!(b > 0, "`div_round(a, b)` requires `b > 0`");
        (a + b / 2) / b
    }

    /// Can support FEC.
    #[must_use]
    pub const fn fec(self) -> bool {
        match self {
            Self::Ms2_5 | Self::Ms5 => false,
            Self::Ms10
            | Self::Ms20
            | Self::Ms40
            | Self::Ms60
            | Self::Ms80
            | Self::Ms100
            | Self::Ms120 => true,
        }
    }
}
