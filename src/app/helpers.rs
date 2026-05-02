use std::sync::{Arc, atomic::AtomicBool};

/// Get value of exit.
#[inline(always)]
pub fn should_exit(exit: &Arc<AtomicBool>) -> bool {
    exit.load(std::sync::atomic::Ordering::Relaxed)
}
