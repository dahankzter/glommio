//! How long a latency-sensitive task queue waits to run once it is woken,
//! as a function of how many busy task queues share the executor.
//!
//! ```bash
//! cargo test --release -p glommio --test flat_scheduler_probe -- --nocapture
//! ```

use glommio::{Latency, LocalExecutorBuilder, Placement, Shares};
use std::{
    future::poll_fn,
    sync::{
        atomic::{AtomicBool, Ordering},
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

fn run(busy: usize, run_for: Duration, cpu: usize) -> Vec<Duration> {
    let slot = Arc::new(Slot {
        armed: AtomicBool::new(false),
        acked: AtomicBool::new(true),
        waker: Mutex::new(None),
        t0: Mutex::new(None),
        lat: Mutex::new(Vec::new()),
    });
    let stop = Arc::new(AtomicBool::new(false));
    let ready = Arc::new(AtomicBool::new(false));

    let ex = {
        let (slot, stop, ready) = (slot.clone(), stop.clone(), ready.clone());
        LocalExecutorBuilder::new(Placement::Fixed(cpu))
            .spawn(move || async move {
                let lat_q = glommio::executor().create_task_queue(
                    Shares::default(),
                    if std::env::var_os("PROBE_LAT_NOTIMPORTANT").is_some() {
                        Latency::NotImportant
                    } else {
                        Latency::Matters(Duration::from_millis(1))
                    },
                    "latency",
                );
                let mut busy_tasks = Vec::new();
                for i in 0..busy {
                    let q = glommio::executor().create_task_queue(
                        Shares::default(),
                        if std::env::var_os("PROBE_BUSY_LATENCY").is_some() {
                            Latency::Matters(Duration::from_millis(1))
                        } else {
                            Latency::NotImportant
                        },
                        &format!("busy-{i}"),
                    );
                    let stop = stop.clone();
                    busy_tasks.push(
                        glommio::spawn_local_into(
                            async move {
                                while !stop.load(Ordering::Relaxed) {
                                    let t = Instant::now();
                                    while t.elapsed() < Duration::from_micros(20) {
                                        std::hint::spin_loop();
                                    }
                                    glommio::yield_if_needed().await;
                                }
                            },
                            q,
                        )
                        .unwrap(),
                    );
                }
                let latency = glommio::spawn_local_into(
                    {
                        let (slot, stop) = (slot.clone(), stop.clone());
                        async move {
                            ready.store(true, Ordering::SeqCst);
                            while !stop.load(Ordering::Relaxed) {
                                poll_fn(|cx| {
                                    if slot.armed.swap(false, Ordering::AcqRel)
                                        || stop.load(Ordering::Relaxed)
                                    {
                                        return Poll::Ready(());
                                    }
                                    *slot.waker.lock().unwrap() = Some(cx.waker().clone());
                                    if slot.armed.swap(false, Ordering::AcqRel) {
                                        return Poll::Ready(());
                                    }
                                    Poll::Pending
                                })
                                .await;
                                if let Some(t0) = slot.t0.lock().unwrap().take() {
                                    slot.lat.lock().unwrap().push(t0.elapsed());
                                }
                                slot.acked.store(true, Ordering::Release);
                            }
                        }
                    },
                    lat_q,
                )
                .unwrap();
                latency.await;
                for t in busy_tasks {
                    t.await;
                }
            })
            .unwrap()
    };

    while !ready.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }
    std::thread::sleep(Duration::from_millis(50));
    let deadline = Instant::now() + run_for;
    let mut x: u64 = 0x9e3779b97f4a7c15;
    while Instant::now() < deadline {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        std::thread::sleep(Duration::from_micros(2000 + x % 3000));
        let waker = loop {
            if let Some(w) = slot.waker.lock().unwrap().take() {
                break w;
            }
            std::hint::spin_loop();
        };
        slot.acked.store(false, Ordering::Release);
        *slot.t0.lock().unwrap() = Some(Instant::now());
        slot.armed.store(true, Ordering::Release);
        waker.wake();
        while !slot.acked.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
    stop.store(true, Ordering::SeqCst);
    slot.armed.store(true, Ordering::Release);
    if let Some(w) = slot.waker.lock().unwrap().take() {
        w.wake();
    }
    ex.join().unwrap();
    let mut lat = slot.lat.lock().unwrap().clone();
    lat.sort();
    lat
}

#[test]
fn flat_scheduler_probe() {
    let cpu: usize = std::env::var("PROBE_CPU").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
    let ms: u64 = std::env::var("PROBE_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(3000);
    let counts: Vec<usize> = std::env::var("PROBE_BUSY")
        .ok()
        .map(|s| s.split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![0, 1, 2, 4, 8, 16]);
    for busy in counts {
        let lat = run(busy, Duration::from_millis(ms), cpu);
        let n = lat.len();
        let q = |p: f64| lat[((n as f64 - 1.0) * p) as usize];
        println!(
            "PROBE busy={busy:<2} n={n:<5} p50={:>10.3?} p90={:>10.3?} p99={:>10.3?} max={:>10.3?}",
            q(0.5),
            q(0.9),
            q(0.99),
            lat[n - 1]
        );
    }
}
