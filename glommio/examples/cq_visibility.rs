//! When does an io_uring timeout completion become visible to a thread that
//! spins in user space? Arms a 1 ms timeout, then spins reading the
//! completion queue without entering the kernel (or with a syscall per spin
//! when SYSCALL is set), and reports how long the completion took to appear.

use io_uring::{opcode, types, IoUring};
use std::time::{Duration, Instant};

fn pin_current_thread(cpu: usize) {
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(cpu, &mut set);
        assert_eq!(libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set), 0);
    }
}

fn main() {
    pin_current_thread(10);
    if std::env::var_os("EVENTFD").is_some() {
        return eventfd_case();
    }
    let syscall = std::env::var_os("SYSCALL").is_some();
    let mut ring = IoUring::new(8).unwrap();
    let mut seen = Vec::new();
    for _ in 0..200 {
        let ts = types::Timespec::new().nsec(1_000_000);
        let sqe = opcode::Timeout::new(&ts).build().user_data(1);
        unsafe { ring.submission().push(&sqe).unwrap() };
        ring.submit().unwrap();
        let start = Instant::now();
        loop {
            if syscall {
                std::thread::yield_now();
            }
            let mut cq = ring.completion();
            cq.sync();
            if cq.next().is_some() {
                break;
            }
            if start.elapsed() > Duration::from_secs(2) {
                break;
            }
            std::hint::spin_loop();
        }
        seen.push(start.elapsed());
    }
    seen.sort();
    let n = seen.len();
    println!(
        "syscall={syscall} 1ms timeout seen after: p50={:?} p90={:?} p99={:?} max={:?}",
        seen[n / 2],
        seen[(n - 1) * 9 / 10],
        seen[(n - 1) * 99 / 100],
        seen[n - 1]
    );
}

fn eventfd_case() {
    let syscall = std::env::var_os("SYSCALL").is_some();
    let efd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC) };
    assert!(efd >= 0);
    let mut ring = IoUring::new(8).unwrap();
    let mut buf = 0u64;
    let mut seen = Vec::new();
    for _ in 0..200 {
        let sqe = opcode::Read::new(types::Fd(efd), &mut buf as *mut u64 as *mut u8, 8)
            .build()
            .user_data(2);
        unsafe { ring.submission().push(&sqe).unwrap() };
        ring.submit().unwrap();
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1));
            let start = Instant::now();
            let one = 1u64;
            unsafe { libc::write(efd, &one as *const u64 as *const libc::c_void, 8) };
            start
        });
        loop {
            if syscall {
                std::thread::yield_now();
            }
            let mut cq = ring.completion();
            cq.sync();
            if cq.next().is_some() {
                break;
            }
            std::hint::spin_loop();
        }
        let done = Instant::now();
        let start = writer.join().unwrap();
        seen.push(done.saturating_duration_since(start));
    }
    seen.sort();
    let n = seen.len();
    println!(
        "syscall={syscall} eventfd write seen after: p50={:?} p90={:?} p99={:?} max={:?}",
        seen[n / 2],
        seen[(n - 1) * 9 / 10],
        seen[(n - 1) * 99 / 100],
        seen[n - 1]
    );
}
