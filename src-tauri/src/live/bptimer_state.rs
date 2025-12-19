use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub type BPTimerEnabledMutex = Arc<AtomicBool>;

pub fn create_bptimer_enabled(initial: bool) -> BPTimerEnabledMutex {
    Arc::new(AtomicBool::new(initial))
}

pub fn set_bptimer_enabled(state: &BPTimerEnabledMutex, enabled: bool) {
    state.store(enabled, Ordering::Relaxed);
}

pub fn is_bptimer_enabled(state: &BPTimerEnabledMutex) -> bool {
    state.load(Ordering::Relaxed)
}
