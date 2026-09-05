// Unless explicitly stated otherwise all files in this repository are licensed
// under the MIT/Apache-2.0 License, at your convenience
//
//! Handle for a timer held in the reactor's ordered map.

use std::time::Instant;

/// Names a timer's place in the ordered map directly.
///
/// Upstream's handle is a bare `u64` and needs a side `HashMap<u64, Instant>`
/// to recover the deadline before it can remove anything, so cancelling costs
/// a hash lookup on top of the tree removal. Carrying the deadline in the
/// handle removes both the map and the lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimerId {
    pub(super) when: Instant,
    /// Distinguishes timers sharing a deadline.
    pub(super) seq: u64,
}
