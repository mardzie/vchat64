use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub fn load_atomic_bool(atomic: &Arc<AtomicBool>) -> bool {
    atomic.load(Ordering::Acquire)
}

pub fn store_atomic_bool(atomic: &Arc<AtomicBool>, val: bool) {
    atomic.store(val, Ordering::Release);
}
