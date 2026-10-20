use crate::SAMPLE_RATE_48K;

const _: () = assert!(FrameSize::Ms20.samples(SAMPLE_RATE_48K) == 960);

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
    const UNIT_US: u32 = 2_500;
    /// 2.5 ms in 1000 ms (1000 / 2.5)
    const UNITS_PER_SECOND: u32 = 1_000_000 / Self::UNIT_US;

    #[must_use]
    #[inline]
    pub(crate) const fn samples(self, sample_rate: u32) -> usize {
        const _: () = assert!(usize::BITS >= u32::BITS, "system must be 32 bits or higher");
        (sample_rate / Self::UNITS_PER_SECOND) as usize * self as usize
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

    #[must_use]
    pub const fn application(self) -> opus::Application {
        match self {
            Self::Ms2_5 | Self::Ms5 => opus::Application::LowDelay,
            Self::Ms10
            | Self::Ms20
            | Self::Ms40
            | Self::Ms60
            | Self::Ms80
            | Self::Ms100
            | Self::Ms120 => opus::Application::Voip,
        }
    }
}
