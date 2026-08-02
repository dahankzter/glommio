# The Network Path — Where the Fat Is

**Date:** 2026-08-02
**Status:** measured — **first genuinely large gap found**
**Method:** raw io_uring floor, glommio over the identical workload, attribute

The task path, the DMA read path and the reactor loop all came back thin: single
digit percentages, device-dominated, nothing worth optimising. The network path
does not.

## Latency: loopback TCP ping-pong

64-byte messages, one request in flight, client on cpu 0 and server on cpu 4
(same L3 domain, so the cache-domain effect measured elsewhere is not a
confound). Three runs, spread under 3%.

| stack | round trip |
|---|---:|
| `std::net` blocking | ~3,250 ns |
| raw io_uring | ~3,660 ns |
| **glommio (parks)** | **~8,000 ns (+120%)** |
| glommio, `spin_before_park(50us)` | ~6,600 ns (+82%) |

Blocking sockets beat both. glommio is **more than twice** the raw floor.

Parking explains only about a third of it: never parking still leaves ~2.9 µs.

## What it is doing

Counters on SQE construction, kernel enters and `Source` registration. These are
process-global so they cover **both** shards; raw is quoted per side.

| | SQEs | `io_uring_enter` | `Source`s |
|---|---:|---:|---:|
| raw io_uring, per side | 2 | 2 | — |
| glommio parking, both sides | 20 | 10 | 10 |
| glommio spinning, both sides | 12 | 8 | 6 |

Even halving glommio's figures, it issues roughly **five SQEs and five kernel
enters per side per round trip where two and two suffice**, and allocates and
registers five `Source`s.

`SockRecv` and `SockSend` counts were **zero**. glommio is not using io_uring's
`Recv`/`Send` for TCP at all.

## Why: the read path is readiness-based, not completion-based

`net/stream.rs`:

```rust
if no_pending_poll {
    if let Some(result) = super::yolo_recv(self.stream.as_raw_fd(), buf) {
        ...
        return Poll::Ready(Ok(result));
    }
}
...
self.source_rx = Some(reactor.poll_read_ready(self.stream.as_raw_fd()));
```

A read is a **direct non-blocking `recv` syscall that bypasses io_uring**, and
only on `EAGAIN` does it register a `PollAdd` and wait. That is an epoll-shaped
design wearing io_uring underneath.

**This is a deliberate trade, not an oversight.** When data is already buffered,
`yolo_recv` returns it in one cheap syscall with no io_uring involvement at all
— cheaper than a completion-based `Recv`. The cost lands when the data is *not*
there: a wasted `recv` returning `EAGAIN`, then a poll registration, then a wake,
then a second `recv`. A ping-pong is that case every single time.

## But the streaming regime is worse, not better

If the trade were sound, the throughput case should favour glommio. Sender never
waits, so the receiver almost always finds data buffered — `yolo_recv`'s best
case. Per message sent:

| stack | per message |
|---|---:|
| `std::net` blocking | **100 ns** |
| glommio | **1,057 ns** |

**10x**, in the regime the design is built for. Whatever is costing ~950 ns per
64-byte send is not the readiness trade — a successful `yolo_send` should be one
syscall.

## What this means

This is the first thing found in this fork's performance work that is large,
reproducible and not explained by hardware. Unlike the read path, where the
device was 91% of the time and there was nothing to give back, here there is a
factor of two on latency and a factor of ten on streaming sends.

It also **relocates the monoio comparison**, which is where this started.
monoio benchmarks are TCP echo servers, not file reads. If glommio loses to
monoio, this is a far likelier explanation than anything in the task or DMA
paths — and it is consistent with monoio being completion-based throughout.

## Before anyone builds something

The obvious move — switch reads and writes to io_uring `Recv`/`Send` — is a
design change to the core of the network stack, and the current design was
presumably chosen for a reason. Measure first, in this order:

1. **Attribute the streaming 950 ns.** That is the clearest signal and the least
   ambiguous case: one successful send should be one syscall. Find out what else
   is happening. It may not be the readiness design at all.
2. **Then** prototype a completion-based read on a branch and measure both
   regimes. A design that wins on latency and loses on throughput is not
   obviously an improvement.
3. Check `rush_dispatch`. `stream.rs` has two commented-out calls with a note
   referring to issue #458 — someone has already been here.

## What this does not cover

Loopback only, one connection, 64-byte messages, TCP only. Not covered: real
NICs, many connections, large messages, UDP, `accept` throughput, and the
receive side of the streaming test in isolation. Loopback exaggerates software
cost because there is no wire time to hide behind — which makes it the right
place to *find* overhead and the wrong place to size its real-world impact.
