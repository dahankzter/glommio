//! What the timer structure costs, in the shape glommio actually uses it.
//!
//! Companion to `timer_ladder`, which answers *how many* timers there are and
//! what happens to them. That measurement said the workload is insert and
//! cancel with essentially no expiry, so the cases here are weighted that way.
//!
//! Everything is driven through the public API, so this file compiles
//! unchanged against any timer implementation and the numbers are comparable
//! across them. Nothing here may reach into a particular wheel's internals —
//! an arm that needs its own probe keeps that probe on its own branch, and
//! those numbers are never set against another arm's.
//!
//! Four cases:
//!
//! 1. **arm** — cost of registering a timer that will not fire.
//! 2. **cancel** — cost of withdrawing it again. Together with (1) this is the
//!    deadline-churn path: a read timeout set and then cancelled when the read
//!    completes.
//! 3. **poll under population** — time per short sleep while N timers sit
//!    pending. A structure that answers "when is the next deadline" by scanning
//!    grows here with N; one that answers structurally does not.
//! 4. **idle gap** — overshoot on a two-second sleep with a populated
//!    structure. A wheel that steps through every elapsed millisecond pays for
//!    the interval; one that steps to the next work does not.
//!
//! Run with:
//! ```bash
//! cargo run --release --features debugging --example timer_bench
//! ```
//!
//! To run it across every arm, use `make timer-arms`.

#[cfg(not(feature = "debugging"))]
fn main() {
    eprintln!(
        "timer_bench needs the counters: cargo run --release --features debugging --example timer_bench"
    );
    std::process::exit(1);
}

#[cfg(feature = "debugging")]
fn main() {
    imp::main()
}

#[cfg(feature = "debugging")]
mod imp {
    use futures_lite::future::poll_once;
    use glommio::{
        timer::{debugging::timer_stats, sleep, Timer},
        LocalExecutor,
    };
    use std::time::{Duration, Instant};

    const POPULATIONS: &[usize] = &[64, 256, 1_024, 4_096];

    /// Sleeps timed in the poll-under-population case. Enough to average out
    /// scheduling noise without making the run long.
    const SLEEPS: usize = 200;

    /// Far enough out that nothing in these cases reaches it.
    const PARKED: Duration = Duration::from_secs(3_600);

    pub fn main() {
        let ex = LocalExecutor::default();
        ex.run(async {
            println!("# timer cost, per executor");
            println!("# {}", std::env::var("ARM").unwrap_or_else(|_| "?".into()));
            println!();

            arm_and_cancel().await;
            poll_under_population().await;
            idle_gap().await;
        });
    }

    /// Hold `n` timers pending, then drop them. Registering and withdrawing are
    /// timed separately because they are different operations on the structure
    /// and a wheel can be good at one and bad at the other.
    async fn arm_and_cancel() {
        println!(
            "{:>7}  {:>12}  {:>12}",
            "timers", "arm ns/op", "cancel ns/op"
        );

        for &n in POPULATIONS {
            let before = timer_stats();

            let mut timers: Vec<Timer> = (0..n).map(|_| Timer::new(PARKED)).collect();

            let started = Instant::now();
            for timer in timers.iter_mut() {
                // The first poll is what registers it with the reactor.
                assert!(poll_once(timer).await.is_none(), "must not fire");
            }
            let armed = started.elapsed();

            let started = Instant::now();
            drop(timers);
            let cancelled = started.elapsed();

            let after = timer_stats();
            assert_eq!(
                after.inserted - before.inserted,
                n as u64,
                "each timer armed exactly once"
            );
            assert_eq!(
                after.cancelled_before_fire - before.cancelled_before_fire,
                n as u64,
                "and was withdrawn, not fired"
            );

            println!(
                "{n:>7}  {:>12.1}  {:>12.1}",
                armed.as_nanos() as f64 / n as f64,
                cancelled.as_nanos() as f64 / n as f64
            );
        }
        println!();
    }

    /// Time a short sleep while `n` timers sit pending.
    ///
    /// The reactor asks for the next deadline on every pass. If that answer
    /// costs a scan of the population, this column climbs with `n`; if it is
    /// answered from the structure, it does not.
    async fn poll_under_population() {
        println!("{:>7}  {:>14}", "pending", "sleep us");

        for &n in POPULATIONS {
            let mut parked: Vec<Timer> = (0..n).map(|_| Timer::new(PARKED)).collect();
            for timer in parked.iter_mut() {
                assert!(poll_once(timer).await.is_none());
            }

            let started = Instant::now();
            for _ in 0..SLEEPS {
                sleep(Duration::from_micros(100)).await;
            }
            let elapsed = started.elapsed();

            drop(parked);

            println!(
                "{n:>7}  {:>14.1}",
                elapsed.as_micros() as f64 / SLEEPS as f64
            );
        }
        println!();
    }

    /// Sleep two seconds with a populated structure, and report how much
    /// longer than two seconds it took.
    ///
    /// Nothing is due in that interval. A structure that walks every elapsed
    /// millisecond pays for the interval itself; one that steps to the next
    /// work pays nothing.
    async fn idle_gap() {
        println!("{:>7}  {:>16}", "pending", "overshoot ms");

        for &n in POPULATIONS {
            let mut parked: Vec<Timer> = (0..n).map(|_| Timer::new(PARKED)).collect();
            for timer in parked.iter_mut() {
                assert!(poll_once(timer).await.is_none());
            }

            let gap = Duration::from_secs(2);
            let started = Instant::now();
            sleep(gap).await;
            let elapsed = started.elapsed();

            drop(parked);

            println!(
                "{n:>7}  {:>16.1}",
                elapsed.saturating_sub(gap).as_secs_f64() * 1_000.0
            );
        }
    }
}
