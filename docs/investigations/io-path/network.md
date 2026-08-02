# The Network Path — Where the Fat Is

**Date:** 2026-08-02
**Status:** latency gap confirmed; an accompanying streaming claim retracted
**Method:** raw io_uring floor, glommio over the identical workload, attribute

The task path, the DMA read path and the reactor loop all came back thin: single
digit percentages, device-dominated, nothing worth optimising. The network path
is the first that is not — on latency. A second claim in this document, about
streaming throughput, turned out to be a benchmark bug and is retracted below.

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

## The streaming claim was wrong — retracted

An earlier version of this document reported streaming sends at 100 ns for
blocking sockets against 1,057 ns for glommio and called it a 10x gap in the
regime the design exists for. **That was a benchmark bug, twice over.**

**First, Nagle.** The raw comparisons left `TCP_NODELAY` off while glommio set
it. With Nagle on, small sends coalesce into the existing segment and return
almost immediately; with it off, each 64-byte send pushes a packet and loopback
delivery happens inline. Setting `NODELAY` on all three collapses the difference:

| per send, `TCP_NODELAY` on all three | |
|---|---:|
| bare `send(MSG_DONTWAIT)` | 1,153 ns |
| + glommio executor and `await` | 1,153 ns (**+0**) |
| + glommio net layer (`write_all`) | 1,280 ns (**+127**) |

**glommio's write path costs 127 ns over the bare syscall, not 950**, and the
executor costs nothing measurable.

Timers inside `poll_write` confirm the shape: 1.00 `poll_write` and 1.00
successful `yolo_send` per message, zero `EAGAIN`, 18 ns in everything after the
syscall. The fast path is one syscall and a few cheap operations, exactly as it
reads.

**Second, backpressure.** The two streaming servers are not comparable either:
the blocking one drains with a cheap `read` syscall while glommio's drains
through its async read path, so the client backs up against a slower reader.
That test measures receive throughput indirectly, not send cost.

The write path is fine. A 10x result should have been suspicious rather than
exciting, and the ladder that disproved it took twenty minutes.

## What this means

The latency result stands: a factor of two, reproducible, not explained by
hardware. That comparison was fair — all three stacks set `TCP_NODELAY`, run the
same workload, and use the same stack on both ends. Unlike the read path, where
the device was 91% of the time and there was nothing to give back, here there is
something to give back.

The streaming result does not stand; see the retraction above.

It also **relocates the monoio comparison**, which is where this started.
monoio benchmarks are TCP echo servers, not file reads. If glommio loses to
monoio, this is a far likelier explanation than anything in the task or DMA
paths — and it is consistent with monoio being completion-based throughout.

## Resolved: it is not the design

[synthesis.md](synthesis.md) has the read ladder. Implementing glommio's
readiness pattern by hand, without glommio, costs **−34 ns** against
completion-based io_uring — the design is free. glommio costs **+2,017 ns over
its own design**, and that figure matches the ~2.2 µs the DMA read path pays at
queue depth 1. It is the per-I/O executor round trip, not the network stack.

The section below was written before that was known; keep it for the reasoning,
ignore its conclusion about redesigning.

## Before anyone builds something

The obvious move — switch reads and writes to io_uring `Recv`/`Send` — is a
design change to the core of the network stack, and the current design was
presumably chosen for a reason. Measure first, in this order:

1. ~~Attribute the streaming 950 ns.~~ **Done, and it evaporated** — see the
   retraction. The write path adds 127 ns; the executor adds nothing.
2. **Attribute the latency gap the same way.** Five SQEs and five enters per side
   per round trip is a count, not an attribution. The write ladder worked; the
   read side has had no equivalent. Find out what each op is and what it costs
   before assuming the readiness design is responsible.
3. **Only then** prototype a completion-based read. Note it would *add* an SQE
   and a kernel enter to the streaming path, which today costs one syscall — so
   it could make throughput worse to make latency better.
4. Check `rush_dispatch`. `stream.rs` has two commented-out calls with a note
   referring to issue #458 — someone has already been here.

## What this does not cover

Loopback only, one connection, 64-byte messages, TCP only. Not covered: real
NICs, many connections, large messages, UDP, `accept` throughput, and the
receive side of the streaming test in isolation. Loopback exaggerates software
cost because there is no wire time to hide behind — which makes it the right
place to *find* overhead and the wrong place to size its real-world impact.
