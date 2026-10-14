use glommio::{LocalExecutorBuilder, Latency, Shares, Placement};
use std::{
    future::poll_fn,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::{Poll, Waker},
    time::{Duration, Instant},
};

struct Slot {
    armed: AtomicBool,
    acked: AtomicBool,
    waker: Mutex<Option<Waker>>,
    t0: Mutex<Option<Instant>>,
    lat: Mutex<Vec<Duration>>,
}

#[test]
fn foreign_latency_probe() {
    let threads: usize = std::env::var("PROBE_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
    let run_for = Duration::from_millis(std::env::var("PROBE_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(3000));
    let ordinary: usize = std::env::var("PROBE_ORDINARY").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let total = threads + ordinary;
    let slots: Vec<Arc<Slot>> = (0..total)
        .map(|_| Arc::new(Slot {
            armed: AtomicBool::new(false),
            acked: AtomicBool::new(true),
            waker: Mutex::new(None),
            t0: Mutex::new(None),
            lat: Mutex::new(Vec::new()),
        }))
        .collect();
    let stop = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(AtomicUsize::new(0));

    let ex_slots = slots.clone();
    let ex_stop = stop.clone();
    let ex_ready = ready.clone();
    let ex = LocalExecutorBuilder::new(Placement::Unbound).spawn(move || async move {
        let lat_q = glommio::executor().create_task_queue(Shares::default(), Latency::Matters(Duration::from_millis(1)), "lat");
        let busy_q = glommio::executor().create_task_queue(Shares::default(), Latency::NotImportant, "busy");
        let mut tasks = Vec::new();
        for (idx, slot) in ex_slots.into_iter().enumerate() {
            let q = if idx < threads { lat_q } else { busy_q };
            let stop = ex_stop.clone();
            let ready = ex_ready.clone();
            tasks.push(glommio::spawn_local_into(async move {
                ready.fetch_add(1, Ordering::SeqCst);
                while !stop.load(Ordering::Relaxed) {
                    poll_fn(|cx| {
                        if slot.armed.swap(false, Ordering::AcqRel) { return Poll::Ready(()); }
                        *slot.waker.lock().unwrap() = Some(cx.waker().clone());
                        if slot.armed.swap(false, Ordering::AcqRel) { return Poll::Ready(()); }
                        if stop.load(Ordering::Relaxed) { return Poll::Ready(()); }
                        Poll::Pending
                    }).await;
                    if let Some(t0) = slot.t0.lock().unwrap().take() {
                        slot.lat.lock().unwrap().push(t0.elapsed());
                    }
                    slot.acked.store(true, Ordering::Release);
                }
            }, q).unwrap());
        }
        let stop2 = ex_stop.clone();
        let busy = glommio::spawn_local_into(async move {
            while !stop2.load(Ordering::Relaxed) {
                let t = Instant::now();
                while t.elapsed() < Duration::from_micros(20) { std::hint::spin_loop(); }
                glommio::yield_if_needed().await;
            }
        }, busy_q).unwrap();
        for t in tasks { t.await; }
        busy.await;
    }).unwrap();

    while ready.load(Ordering::SeqCst) < total { std::thread::yield_now(); }
    std::thread::sleep(Duration::from_millis(50));
    let deadline = Instant::now() + run_for;
    let mut handles = Vec::new();
    for (i, slot) in slots.iter().cloned().enumerate() {
        handles.push(std::thread::spawn(move || {
            let mut x: u64 = 0x9e3779b97f4a7c15 ^ (i as u64 + 1);
            while Instant::now() < deadline {
                x ^= x << 13; x ^= x >> 7; x ^= x << 17;
                let pause = Duration::from_micros(x % 200);
                let t = Instant::now();
                while t.elapsed() < pause { std::hint::spin_loop(); }
                let waker = loop {
                    if let Some(w) = slot.waker.lock().unwrap().take() { break w; }
                    if Instant::now() > deadline + Duration::from_secs(2) { return; }
                    std::hint::spin_loop();
                };
                slot.acked.store(false, Ordering::Release);
                *slot.t0.lock().unwrap() = Some(Instant::now());
                slot.armed.store(true, Ordering::Release);
                waker.wake();
                while !slot.acked.load(Ordering::Acquire) { std::hint::spin_loop(); }
            }
        }));
    }
    for h in handles { h.join().unwrap(); }
    stop.store(true, Ordering::SeqCst);
    for slot in &slots {
        slot.armed.store(true, Ordering::Release);
        if let Some(w) = slot.waker.lock().unwrap().take() { w.wake(); }
    }
    ex.join().unwrap();
    let mut all: Vec<Duration> = slots[..threads].iter().flat_map(|s| s.lat.lock().unwrap().clone()).collect();
    all.sort();
    let n = all.len();
    let q = |p: f64| all[((n as f64 - 1.0) * p) as usize];
    let over = |ms: u64| all.iter().filter(|d| **d > Duration::from_millis(ms)).count();
    println!("PROBE n={n} p50={:?} p99={:?} p999={:?} max={:?} >5ms={} >50ms={}", q(0.5), q(0.99), q(0.999), all[n - 1], over(5), over(50));
}
