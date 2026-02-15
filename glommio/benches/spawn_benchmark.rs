// Copyright 2024 Glommio Project Authors. Licensed under Apache-2.0.

//! Benchmark spawn_local performance to measure task allocation overhead
//!
//! This benchmark specifically targets the arena allocator improvement:
//! Goal: Reduce spawn_local latency from ~80ns to ~20ns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glommio::{spawn_local, LocalExecutor};
use std::time::Duration;

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(50) // Reduced from 500 to avoid OOM (each sample creates new executor with 1MB arena)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5))
}

/// Benchmark spawning tasks that complete immediately
fn bench_spawn_immediate(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_immediate");

    for count in [1, 10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                LocalExecutor::default().run(async {
                    for _ in 0..count {
                        spawn_local(async {
                            black_box(42);
                        })
                        .detach();
                    }
                })
            });
        });
    }

    group.finish();
}

/// Benchmark spawning tasks that actually await
fn bench_spawn_with_await(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_with_await");

    for count in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                LocalExecutor::default().run(async {
                    let mut handles = Vec::new();
                    for i in 0..count {
                        handles.push(spawn_local(async move {
                            black_box(i);
                        }));
                    }
                    for handle in handles {
                        handle.await;
                    }
                })
            });
        });
    }

    group.finish();
}

/// Measure pure spawn latency (spawn + detach, no actual work)
fn bench_spawn_latency(c: &mut Criterion) {
    c.bench_function("spawn_latency", |b| {
        b.iter(|| {
            LocalExecutor::default().run(async {
                spawn_local(async {
                    black_box(1);
                })
                .detach();
            })
        });
    });
}

/// Benchmark spawn throughput - how many tasks/sec can we spawn
fn bench_spawn_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_throughput");

    for count in [100, 1000] {
        group.throughput(criterion::Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                LocalExecutor::default().run(async {
                    for _ in 0..count {
                        spawn_local(async {}).detach();
                    }
                })
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = spawn_benches;
    config = criterion_config();
    targets = bench_spawn_immediate, bench_spawn_with_await, bench_spawn_latency, bench_spawn_throughput
}
criterion_main!(spawn_benches);
