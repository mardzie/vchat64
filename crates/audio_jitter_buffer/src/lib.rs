use std::{num::NonZeroUsize, time::Duration};

use audio_common::frame_size::FrameSize;
pub use entry::Entry;

mod entry;

pub mod error;

pub type SequenceNumber = u64;

#[derive(Debug)]
pub struct JitterBuf<T>
where
    T: Clone,
{
    inner: Vec<Option<Entry<T>>>,
    last_popped_seq: SequenceNumber,
    newest_pushed_seq: SequenceNumber,
}

impl<T> JitterBuf<T>
where
    T: Clone,
{
    const DEFAULT_CAPACTITY_MULTIPLIER: NonZeroUsize =
        NonZeroUsize::new(4).expect("default capacity multiplier must be non zero");

    #[must_use]
    pub fn new(
        target_delay: Duration,
        frame_size: FrameSize,
    ) -> Result<Self, error::ConstructionError> {
        Self::with_multiplier(target_delay, frame_size, Self::DEFAULT_CAPACTITY_MULTIPLIER)
    }

    pub fn with_multiplier(
        target_delay: Duration,
        frame_size: FrameSize,
        // To multiply the necessary capacity for bursts of incoming packets.
        capacity_multiplier: NonZeroUsize,
    ) -> Result<Self, error::ConstructionError> {
        let capacity = target_delay
            .as_nanos()
            .div_ceil(frame_size.duration().as_nanos());

        Ok(Self {
            inner: vec![None; capacity as usize * capacity_multiplier.get()],
            last_popped_seq: 0,
            newest_pushed_seq: 0,
        })
    }

    /// Push a new `Entry` to the `JitterBuf`.
    #[inline]
    pub fn push(&mut self, entry: Entry<T>) -> Result<(), error::PushError> {
        let seq = entry.seq();
        if seq <= self.last_popped_seq() {
            return Err(error::PushError::LatePacket {
                seq,
                next_expected: self.newest_pushed_seq + 1,
            });
        }

        let index = self.index_from_seq(seq);
        if index >= self.current_pop_index() {
            return Err(error::PushError::TooFarAhead {
                seq,
                window_end: self.last_popped_seq,
            });
        }

        let value = self.inner.get_mut(index).expect("index must be in bounds");
        if let Some(old_entry) = value
            && &entry == old_entry
        {
            return Err(error::PushError::Duplicate { seq });
        }
        *value = Some(entry);
        self.newest_pushed_seq = self.newest_pushed_seq().max(seq);

        Ok(())
    }

    /// Pop the next `Entry` from the `JitterBuf`.
    ///
    /// Advances the pointer by one each pop independant of the `Option` variant.
    #[must_use]
    #[inline]
    pub fn pop(&mut self) -> Option<Entry<T>> {
        let current_index = self.current_pop_index();
        // The `current_index` must be in bounds. `current_index()` guarantees that.
        let value = self
            .inner
            .get_mut(current_index)
            .expect("index must be in bounds");
        self.last_popped_seq += 1;
        let entry = std::mem::take(value);

        #[cfg(debug_assertions)]
        {
            match &entry {
                Some(entry) => debug_assert_eq!(entry.seq(), self.last_popped_seq),
                _ => (),
            };
        }

        entry
    }

    /// The last popped sequence number.
    #[must_use]
    #[inline]
    pub fn last_popped_seq(&self) -> u64 {
        self.last_popped_seq
    }

    /// The newest pushed sequence number.
    #[must_use]
    #[inline]
    pub fn newest_pushed_seq(&self) -> u64 {
        self.newest_pushed_seq
    }

    /// Returns the current index for `inner`.
    ///
    /// The index is always valid.
    fn current_pop_index(&self) -> usize {
        self.index_from_seq(self.last_popped_seq + 1)
    }

    /// Returns the index of the `sequence_number`.
    ///
    /// The index is always valid.
    fn index_from_seq(&self, seq: u64) -> usize {
        let index = (usize::try_from(seq).expect("can not convert u64 to usize") + 1)
            % self.inner.capacity();
        debug_assert!(self.inner.get(index).is_some());

        index
    }
}

impl<T> Default for JitterBuf<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new(Duration::from_millis(100), FrameSize::default()).expect("default must be valid")
    }
}
