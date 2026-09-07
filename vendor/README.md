# Vendored dependencies

Temporary, and only on `arm/b-bitwheel`.

`bitwheel/` is bitwheel 0.6.0 (MIT OR Apache-2.0) with two patches, so the arm
can be measured at all. Both are marked `PATCHED (glommio timer comparison)`
in the source and reported upstream as
[Abso1ut3Zer0/bitwheel#18](https://github.com/Abso1ut3Zer0/bitwheel/issues/18):

- `cancel` used an unchecked removal justified by "a timer whose deadline is
  still in the future cannot have fired". `poll_tick` fires a whole gear slot
  when the tick divides its span, so that is false, and cancelling such a timer
  reached `hint::unreachable_unchecked` — undefined behaviour in release. Now
  uses the crate's own `try_remove`.
- `len` was decremented in `cancel` but not when a timer fired, so it
  over-reported permanently.
- `insert` clamped `delay` to at least one tick but computed the target slot
  from the unclamped deadline, so a timer due inside the current tick landed in
  the current slot -- which `compute_gear_min_fire` skips. It stayed invisible
  to `duration_until_next` for a whole gear revolution, making a 100us sleep
  take 64ms. Measured 64,370us before the one-line fix and 1,128us after.

Neither patch touches the structure being measured. If the arm is not adopted
this directory goes away; if it is, the dependency comes from crates.io once
the fixes are released, and never from here.
