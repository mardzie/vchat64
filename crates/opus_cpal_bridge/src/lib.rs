#![warn(clippy::pedantic)]

pub use opus::Bitrate;

use crate::opus_codec::frame_size::FrameSize;

mod macros;

pub mod error;
pub mod opus_codec;
pub mod resampler;
pub mod stream;

const SAMPLE_RATE_48K: u32 = 48_000;
const _: () = assert!(
    cpal::SAMPLE_RATE_48K == SAMPLE_RATE_48K,
    "48 K sample rate must be the same!"
);
const FRAME_SIZE: FrameSize = FrameSize::Ms20;

/// Clamp but without branches.
///
/// **DO NOT USE** `.clamp()` is very likely faster.
#[must_use]
#[inline]
pub const fn branch_free_clamp(value: f32, min: f32, max: f32) -> f32 {
    debug_assert!(min <= max);

    let v = select(value < min, min, value);
    select(v > max, max, v)
}

/// Returns `a` if `c` is `true` and `b` otherwise.
///
/// It uses branchless programming to achieve this.
#[must_use]
const fn select(c: bool, a: f32, b: f32) -> f32 {
    let m = (c as u32).wrapping_neg();
    f32::from_bits(b.to_bits() ^ ((a.to_bits() ^ b.to_bits()) & m))
}

#[cfg(test)]
mod branch_free_clamp_test {
    use crate::branch_free_clamp;

    #[test]
    fn in_range() {
        let value = 0.0;
        let min = -10.0;
        let max = 10.0;
        assert_eq!(branch_free_clamp(value, min, max), value);
    }

    #[test]
    fn min_max_equals() {
        let value = 10.0;
        assert_eq!(branch_free_clamp(value, value, value), value);
    }

    #[test]
    fn low() {
        let value = -11.0;
        let min = -10.0;
        let max = 10.0;
        assert_eq!(branch_free_clamp(value, min, max), min);
    }

    #[test]
    fn high() {
        let value = 11.0;
        let min = -10.0;
        let max = 10.0;
        assert_eq!(branch_free_clamp(value, min, max), max);
    }

    #[test]
    fn on_high_edge() {
        let value = 10.0;
        let min = -10.0;
        let max = 10.0;
        assert_eq!(branch_free_clamp(value, min, max), value);
    }

    #[test]
    fn on_low_edge() {
        let value = -10.0;
        let min = -10.0;
        let max = 10.0;
        assert_eq!(branch_free_clamp(value, min, max), value);
    }
}
