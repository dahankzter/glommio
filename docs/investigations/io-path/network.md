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

## Vectored writes: measured, and fixed (2026-08-24)

`TcpStream` did not override `poll_write_vectored`, so it inherited the
futures-io default, which writes **only the first slice**. A server sending a
response as status line, headers and body therefore paid three `send` calls,
and with `TCP_NODELAY` set -- which a latency-sensitive server does set -- put
up to three segments on the wire for one response.

`glommio/examples/writev_ladder.rs`, 100k responses, loopback, `OutSegs` from
`/proc/net/snmp` (host-wide, so it counts the drain server's ACKs too -- the
ratio is the signal, not the absolute):

| 13-byte body | per response | segments |
|---|---:|---:|
| three `write_all` calls | 6,760 ns | 5.50 |
| one `write_vectored` | **1,782 ns** | **1.88** |
| concatenate, then one `write_all` | 1,988 ns | 1.86 |

| 64 KiB body | per response | segments |
|---|---:|---:|
| three `write_all` calls | 9,287 ns | 6.61 |
| one `write_vectored` | **6,726 ns** | 3.00 |
| concatenate, then one `write_all` | 7,906 ns | 3.00 |

**Three writes to one is worth 3.8x on a small response**, and the segment
count drops by the same factor -- which is the part that matters off loopback,
where a packet costs a wire traversal rather than a memcpy.

**Vectoring versus concatenating is a wash on small bodies** (inside run-to-run
noise) and worth ~1,180 ns on a 64 KiB body, which is a 64 KiB copy at memory
bandwidth. That is the honest size of that particular win: it is the copy, and
it scales with the body.

An earlier estimate in conversation put the saving at ~2 µs by multiplying a
per-send ping-pong figure by three. That was wrong in both directions -- it
overstated the syscall component and ignored the segment count, which is the
larger effect.

## Accept: multishot measured and rejected (2026-08-24)

`accept` speculates like the read path: `yolo_accept` calls `accept` and only
falls back to an io_uring `Accept` on `EAGAIN`. So under churn the ring is not
involved, and multishot accept would be replacing a syscall rather than adding
one. Whether that is a win is a measurement, and it went three rounds before
giving a trustworthy answer.

`glommio/examples/accept_ladder.rs`, 10,240 connections, per connection:

| rung | wall | **cpu** |
|---|---:|---:|
| toggle `O_NONBLOCK` per accept (what glommio did) | 17,023 ns | 1,652 ns |
| listener non-blocking once | 7,538 ns | 1,326 ns |
| multishot accept, zero syscalls per connection | 6,492 ns | **1,823 ns** |

**Multishot accept costs more CPU than the plain syscall it would replace.**
The CQE drain and the kernel enters that go with it are not cheaper than one
non-blocking `accept`. There is no case here for the reactor surgery multishot
would need — a `Source` that survives its own completion — and this closes the
"accept throughput under churn" item [scaling.md](scaling.md) left open.

### Two ways the measurement lied first

**The wall column is not the answer.** Wall clock is bounded by the connecting
thread, so every rung finishes at the client's pace no matter what the server
spends. Only `CLOCK_THREAD_CPUTIME_ID` on the accepting thread separates them.

**Timing only the drain loop credited multishot with 3 ns per connection.**
With multishot armed the kernel accepts as connections *arrive*, so the work
happens before the timed section starts. Timing the whole round — fill and
drain — puts it back on the measured path. A rung that looks 500x better than
the next one is a bug in the probe, not a result.

And the toggle rung's wall figure is inflated for a third reason: it spins on
`EAGAIN` at four syscalls a turn where the others spin at one. That inflation
is exactly why the in-situ number below matters more than the ladder.

### What was kept

The per-accept `O_NONBLOCK` toggle is gone: listeners are put in non-blocking
mode when they are constructed, so `yolo_accept` is one syscall rather than
four (`F_GETFL`, `F_SETFL`, `accept`, `F_SETFL`).

In isolation that is worth ~326 ns of CPU per accept. **Inside glommio it is
not measurable**: the accept loop costs 1,838 ns of CPU per connection before
and 1,805 ns after, with the runs overlapping. It is kept because it is
strictly less work and removes three syscalls, not because a number moved.

It does buy an invariant: a listener that reaches `yolo_accept` in blocking
mode would park the whole executor inside `accept`. All three construction
sites set the flag, and a `debug_assert` fails loudly rather than hanging if a
fourth ever appears.

## What this does not cover

Loopback only, one connection, 64-byte messages, TCP only. Not covered: real
NICs, many connections, large messages, UDP, `accept` throughput, and the
receive side of the streaming test in isolation. Loopback exaggerates software
cost because there is no wire time to hide behind — which makes it the right
place to *find* overhead and the wrong place to size its real-world impact.
