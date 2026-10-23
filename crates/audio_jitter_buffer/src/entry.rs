/// A `JitterBuf` entry.
///
/// The `number` is the only
#[derive(Debug, Clone)]
pub struct Entry<T>
where
    T: Clone,
{
    sequence_number: u64,
    timestamp: u64,
    inner: T,
}

impl<T> Entry<T>
where
    T: Clone,
{
    #[must_use]
    pub fn new(sequence_number: u64, timestamp: u64, inner: T) -> Self {
        Self {
            sequence_number,
            timestamp,
            inner,
        }
    }

    /// The sequence number of this `Entry`.
    #[must_use]
    #[inline]
    pub fn seq(&self) -> u64 {
        self.sequence_number
    }

    /// The timestamp of this `Entry`.
    #[must_use]
    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Get a reference to `T`.
    #[must_use]
    #[inline]
    pub fn get(&self) -> &T {
        &self.inner
    }

    #[must_use]
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> PartialEq for Entry<T>
where
    T: Clone,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.sequence_number == other.sequence_number
    }
}
impl<T> Eq for Entry<T> where T: Clone {}

impl<T> Ord for Entry<T>
where
    T: Clone,
{
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}
impl<T> PartialOrd for Entry<T>
where
    T: Clone,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
