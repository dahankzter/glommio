//! Temporary instrumentation for the wake latency investigation.
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Instant,
};

static ON: AtomicBool = AtomicBool::new(false);
static EVENTS: Mutex<Vec<(Instant, u8, u32)>> = Mutex::new(Vec::new());

#[doc(hidden)]
pub fn enable(on: bool) {
    ON.store(on, Ordering::SeqCst);
}

#[doc(hidden)]
pub fn record(tag: u8, arg: u32) {
    if ON.load(Ordering::Relaxed) {
        EVENTS.lock().unwrap().push((Instant::now(), tag, arg));
    }
}

#[doc(hidden)]
pub fn take() -> Vec<(Instant, u8, u32)> {
    std::mem::take(&mut *EVENTS.lock().unwrap())
}
