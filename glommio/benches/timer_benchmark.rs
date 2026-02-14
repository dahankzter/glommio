// Micro-benchmarks comparing BTreeMap vs TimingWheel timer implementations
//
// Run with: cargo bench --bench timer_benchmark --features timing-wheel

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glommio::timer::timing_wheel::TimingWheel;
use glommio::timer::staged_wheel::StagedWheel;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::task::{Wake, Waker};
use std::time::{Duration, Instant};

// ============================================================================
// Dummy waker for benchmarks
// ============================================================================

struct DummyWaker;

impl Wake for DummyWaker {
    fn wake(self: Arc<Self>) {}
}

fn dummy_waker() -> Waker {
    Arc::new(DummyWaker).into()
}

// ============================================================================
// BTreeMap baseline implementation (current Glommio approach)
// ============================================================================

struct BTreeMapTimers {
    timers: BTreeMap<(Instant, u64), Waker>,
    next_id: u64,
}

impl BTreeMapTimers {
    fn new() -> Self {
        Self {
            timers: BTreeMap::new(),
            next_id: 0,
        }
    }

    fn insert(&mut self, expires_at: Instant, waker: Waker) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.timers.insert((expires_at, id), waker);
        id
    }

    fn remove(&mut self, expires_at: Instant, id: u64) -> bool {
        self.timers.remove(&(expires_at, id)).is_some()
    }

    fn process_expired(&mut self, now: Instant) -> Vec<Waker> {
        // This is the expensive operation in BTreeMap approach
        let ready = self.timers.split_off(&(now, 0));
        std::mem::replace(&mut self.timers, ready)
            .into_iter()
            .map(|(_, waker)| waker)
            .collect()
    }

    fn len(&self) -> usize {
        self.timers.len()
    }
}

// ============================================================================
// Benchmark: Insert operations
// ============================================================================

fn bench_insert_btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_insert_btreemap");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut timers = BTreeMapTimers::new();
                let start = Instant::now();

                for i in 0..size {
                    let expires_at = start + Duration::from_millis(i as u64);
                    timers.insert(expires_at, dummy_waker());
                }

                black_box(timers);
            });
        });
    }

    group.finish();
}

fn bench_insert_timing_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_insert_timing_wheel");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let start = Instant::now();
                let mut wheel = TimingWheel::new_at(start);

                for i in 0..size {
                    let expires_at = start + Duration::from_millis(i as u64);
                    wheel.insert(expires_at, dummy_waker());
                }

                black_box(wheel);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark: Remove operations
// ============================================================================

fn bench_remove_btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_remove_btreemap");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: insert timers
                    let mut timers = BTreeMapTimers::new();
                    let start = Instant::now();
                    let mut ids = Vec::new();

                    for i in 0..size {
                        let expires_at = start + Duration::from_millis(i as u64);
                        let id = timers.insert(expires_at, dummy_waker());
                        ids.push((expires_at, id));
                    }

                    (timers, ids)
                },
                |(mut timers, ids)| {
                    // Benchmark: remove all timers
                    for (expires_at, id) in ids {
                        black_box(timers.remove(expires_at, id));
                    }
                    black_box(timers);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_remove_timing_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_remove_timing_wheel");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: insert timers
                    let start = Instant::now();
                    let mut wheel = TimingWheel::new_at(start);
                    let mut ids = Vec::new();

                    for i in 0..size {
                        let expires_at = start + Duration::from_millis(i as u64);
                        let id = wheel.insert(expires_at, dummy_waker());
                        ids.push(id);
                    }

                    (wheel, ids)
                },
                |(mut wheel, ids)| {
                    // Benchmark: remove all timers
                    for id in ids {
                        black_box(wheel.remove(id));
                    }
                    black_box(wheel);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark: Process expired timers
// ============================================================================

fn bench_process_expired_btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_process_btreemap");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: insert timers that will expire
                    let mut timers = BTreeMapTimers::new();
                    let start = Instant::now();

                    for i in 0..size {
                        let expires_at = start + Duration::from_millis(i as u64);
                        timers.insert(expires_at, dummy_waker());
                    }

                    (timers, start + Duration::from_millis(size as u64))
                },
                |(mut timers, now)| {
                    // Benchmark: process all expired timers
                    let expired = timers.process_expired(now);
                    black_box(expired);
                    black_box(timers);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_process_expired_timing_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_process_timing_wheel");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: insert timers that will expire
                    let start = Instant::now();
                    let mut wheel = TimingWheel::new_at(start);

                    for i in 0..size {
                        let expires_at = start + Duration::from_millis(i as u64);
                        wheel.insert(expires_at, dummy_waker());
                    }

                    (wheel, start + Duration::from_millis(size as u64))
                },
                |(mut wheel, now)| {
                    // Benchmark: advance time and drain expired
                    wheel.advance_to(now);
                    let expired: Vec<_> = wheel.drain_expired().collect();
                    black_box(expired);
                    black_box(wheel);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark: Mixed workload (realistic usage)
// ============================================================================

fn bench_mixed_workload_btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_mixed_btreemap");

    for concurrent in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*concurrent as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent),
            concurrent,
            |b, &concurrent| {
                b.iter_batched(
                    || {
                        let mut timers = BTreeMapTimers::new();
                        let start = Instant::now();

                        // Setup: insert many timers at different delays
                        for i in 0..concurrent {
                            let expires_at = start + Duration::from_millis(i as u64 % 1000);
                            timers.insert(expires_at, dummy_waker());
                        }

                        (timers, start)
                    },
                    |(mut timers, start)| {
                        // Benchmark: mixed operations
                        // 1. Process some expired (first 100ms)
                        let now = start + Duration::from_millis(100);
                        let _expired = timers.process_expired(now);

                        // 2. Insert some new timers
                        for i in 0..100 {
                            let expires_at = now + Duration::from_millis(i);
                            timers.insert(expires_at, dummy_waker());
                        }

                        // 3. Process more expired (next 100ms)
                        let now = now + Duration::from_millis(100);
                        let _expired = timers.process_expired(now);

                        black_box(timers);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_mixed_workload_timing_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_mixed_timing_wheel");

    for concurrent in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*concurrent as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent),
            concurrent,
            |b, &concurrent| {
                b.iter_batched(
                    || {
                        let start = Instant::now();
                        let mut wheel = TimingWheel::new_at(start);

                        // Setup: insert many timers at different delays
                        for i in 0..concurrent {
                            let expires_at = start + Duration::from_millis(i as u64 % 1000);
                            wheel.insert(expires_at, dummy_waker());
                        }

                        (wheel, start)
                    },
                    |(mut wheel, start)| {
                        // Benchmark: mixed operations
                        // 1. Advance and process some expired (first 100ms)
                        let now = start + Duration::from_millis(100);
                        wheel.advance_to(now);
                        let _expired: Vec<_> = wheel.drain_expired().collect();

                        // 2. Insert some new timers
                        for i in 0..100 {
                            let expires_at = now + Duration::from_millis(i);
                            wheel.insert(expires_at, dummy_waker());
                        }

                        // 3. Advance and process more expired (next 100ms)
                        let now = now + Duration::from_millis(100);
                        wheel.advance_to(now);
                        let _expired: Vec<_> = wheel.drain_expired().collect();

                        black_box(wheel);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: Single operation latency (P99-focused)
// ============================================================================

fn bench_single_insert_btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_single_insert_btreemap");
    group.measurement_time(Duration::from_secs(10));

    for existing in [0, 1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(existing),
            existing,
            |b, &existing| {
                let mut timers = BTreeMapTimers::new();
                let start = Instant::now();

                // Pre-populate with existing timers
                for i in 0..existing {
                    let expires_at = start + Duration::from_millis(i as u64);
                    timers.insert(expires_at, dummy_waker());
                }

                let mut counter = 0u64;
                b.iter(|| {
                    // Benchmark: single insert into populated structure
                    let expires_at = start + Duration::from_millis(counter % 10000);
                    counter += 1;
                    black_box(timers.insert(expires_at, dummy_waker()));
                });
            },
        );
    }

    group.finish();
}

fn bench_single_insert_timing_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_single_insert_timing_wheel");
    group.measurement_time(Duration::from_secs(10));

    for existing in [0, 1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(existing),
            existing,
            |b, &existing| {
                let start = Instant::now();
                let mut wheel = TimingWheel::new_at(start);

                // Pre-populate with existing timers
                for i in 0..existing {
                    let expires_at = start + Duration::from_millis(i as u64);
                    wheel.insert(expires_at, dummy_waker());
                }

                let mut counter = 0u64;
                b.iter(|| {
                    // Benchmark: single insert into populated structure
                    let expires_at = start + Duration::from_millis(counter % 10000);
                    counter += 1;
                    black_box(wheel.insert(expires_at, dummy_waker()));
                });
            },
        );
    }

    group.finish();
}

fn bench_insert_staged_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_insert_staged_wheel");

    for count in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let start = Instant::now();
                    (StagedWheel::new_at(start), start)
                },
                |(mut wheel, start)| {
                    for i in 0..count {
                        let expires_at = start + Duration::from_millis(i as u64);
                        black_box(wheel.insert(expires_at, dummy_waker()));
                    }
                    black_box(wheel);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_remove_staged_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_remove_staged_wheel");

    for count in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let start = Instant::now();
                    let mut wheel = StagedWheel::new_at(start);
                    let mut ids = Vec::with_capacity(count);

                    for i in 0..count {
                        let expires_at = start + Duration::from_millis(i as u64);
                        let id = wheel.insert(expires_at, dummy_waker());
                        ids.push(id);
                    }

                    (wheel, ids)
                },
                |(mut wheel, ids)| {
                    for id in ids {
                        black_box(wheel.remove(id));
                    }
                    black_box(wheel);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_process_expired_staged_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_process_expired_staged_wheel");

    for count in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let start = Instant::now();
                    let mut wheel = StagedWheel::new_at(start);

                    for i in 0..count {
                        let expires_at = start + Duration::from_millis(i as u64);
                        wheel.insert(expires_at, dummy_waker());
                    }

                    (wheel, start)
                },
                |(mut wheel, start)| {
                    let now = start + Duration::from_millis(count as u64);
                    wheel.advance_to(now);
                    let expired: Vec<_> = wheel.drain_expired().collect();
                    black_box(expired);
                    black_box(wheel);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_mixed_workload_staged_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_mixed_workload_staged_wheel");

    for count in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let start = Instant::now();
                    (StagedWheel::new_at(start), start)
                },
                |(mut wheel, start)| {
                    let mut ids = Vec::with_capacity(count / 2);

                    for i in 0..count {
                        let expires_at = start + Duration::from_millis(i as u64);
                        let id = wheel.insert(expires_at, dummy_waker());

                        if i % 2 == 0 {
                            ids.push(id);
                        }
                    }

                    for id in ids {
                        black_box(wheel.remove(id));
                    }

                    let now = start + Duration::from_millis(count as u64);
                    wheel.advance_to(now);
                    let expired: Vec<_> = wheel.drain_expired().collect();
                    black_box(expired);
                    black_box(wheel);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_single_insert_staged_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_single_insert_staged_wheel");
    group.measurement_time(Duration::from_secs(10));

    for existing in [0, 1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(existing),
            existing,
            |b, &existing| {
                let start = Instant::now();
                let mut wheel = StagedWheel::new_at(start);

                // Pre-populate with existing timers
                for i in 0..existing {
                    let expires_at = start + Duration::from_millis(i as u64);
                    wheel.insert(expires_at, dummy_waker());
                }

                let mut counter = 0u64;
                b.iter(|| {
                    // Benchmark: single insert into populated structure
                    let expires_at = start + Duration::from_millis(counter % 10000);
                    counter += 1;
                    black_box(wheel.insert(expires_at, dummy_waker()));
                });
            },
        );
    }

    group.finish();
}

fn bench_churn_staged_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_churn_staged_wheel");

    for concurrent in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*concurrent as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent),
            concurrent,
            |b, &concurrent| {
                b.iter_batched(
                    || {
                        // Setup: pre-populate with timers
                        let start = Instant::now();
                        let mut wheel = StagedWheel::new_at(start);

                        for i in 0..concurrent {
                            let expires_at = start + Duration::from_millis(i as u64 + 1000);
                            wheel.insert(expires_at, dummy_waker());
                        }

                        (wheel, start)
                    },
                    |(mut wheel, start)| {
                        // Benchmark: churn 1000 timers (insert + immediate cancel)
                        for i in 0..1000 {
                            let expires_at = start + Duration::from_millis(i + 500);
                            let id = wheel.insert(expires_at, dummy_waker());
                            // Immediately cancel (simulates ACK arriving before timeout)
                            black_box(wheel.remove(id));
                        }

                        black_box(wheel);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================================
// Register all benchmarks
// ============================================================================

criterion_group!(
    insert_benches,
    bench_insert_btreemap,
    bench_insert_timing_wheel,
    bench_insert_staged_wheel,
    bench_single_insert_btreemap,
    bench_single_insert_timing_wheel,
    bench_single_insert_staged_wheel,
);

criterion_group!(
    remove_benches,
    bench_remove_btreemap,
    bench_remove_timing_wheel,
    bench_remove_staged_wheel,
);

criterion_group!(
    process_benches,
    bench_process_expired_btreemap,
    bench_process_expired_timing_wheel,
    bench_process_expired_staged_wheel,
);

criterion_group!(
    mixed_benches,
    bench_mixed_workload_btreemap,
    bench_mixed_workload_timing_wheel,
    bench_mixed_workload_staged_wheel,
);

// ============================================================================
// Benchmark: Churn (insert + cancel pattern)
// ============================================================================
// This simulates message queue behavior where ACKs arrive before timeout:
// timers are inserted but then cancelled before they expire. Tests the
// "hot path" of continuous insert/remove without the process_expired overhead.

fn bench_churn_btreemap(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_churn_btreemap");

    for concurrent in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*concurrent as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent),
            concurrent,
            |b, &concurrent| {
                b.iter_batched(
                    || {
                        // Setup: pre-populate with timers
                        let mut timers = BTreeMapTimers::new();
                        let start = Instant::now();

                        for i in 0..concurrent {
                            let expires_at = start + Duration::from_millis(i as u64 + 1000);
                            timers.insert(expires_at, dummy_waker());
                        }

                        (timers, start)
                    },
                    |(mut timers, start)| {
                        // Benchmark: churn 1000 timers (insert + immediate cancel)
                        for i in 0..1000 {
                            let expires_at = start + Duration::from_millis(i + 500);
                            let id = timers.insert(expires_at, dummy_waker());
                            // Immediately cancel (simulates ACK arriving before timeout)
                            black_box(timers.remove(expires_at, id));
                        }

                        black_box(timers);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_churn_timing_wheel(c: &mut Criterion) {
    let mut group = c.benchmark_group("timer_churn_timing_wheel");

    for concurrent in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*concurrent as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent),
            concurrent,
            |b, &concurrent| {
                b.iter_batched(
                    || {
                        // Setup: pre-populate with timers
                        let start = Instant::now();
                        let mut wheel = TimingWheel::new_at(start);

                        for i in 0..concurrent {
                            let expires_at = start + Duration::from_millis(i as u64 + 1000);
                            wheel.insert(expires_at, dummy_waker());
                        }

                        (wheel, start)
                    },
                    |(mut wheel, start)| {
                        // Benchmark: churn 1000 timers (insert + immediate cancel)
                        for i in 0..1000 {
                            let expires_at = start + Duration::from_millis(i + 500);
                            let id = wheel.insert(expires_at, dummy_waker());
                            // Immediately cancel (simulates ACK arriving before timeout)
                            black_box(wheel.remove(id));
                        }

                        black_box(wheel);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================================
// Register all benchmarks
// ============================================================================

criterion_group!(
    churn_benches,
    bench_churn_btreemap,
    bench_churn_timing_wheel,
    bench_churn_staged_wheel,
);

criterion_main!(
    insert_benches,
    remove_benches,
    process_benches,
    mixed_benches,
    churn_benches
);
