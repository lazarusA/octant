//! Download progress of the running block fetches, shared by every reader:
//! each fetch adds the chunks its window needs (`plan`) and each chunk as it
//! arrives (`arrived`), with the network bytes it took (`bytes`). Counted
//! from the first fetch until none runs (`BlockPrefetcher`, `reset`), so the
//! status bar shows one figure for every fetch in flight.

use std::sync::atomic::{AtomicU64, Ordering};

static PLANNED: AtomicU64 = AtomicU64::new(0);
static ARRIVED: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

/// What the running fetches downloaded so far.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Snapshot {
    /// Network bytes.
    pub bytes: u64,
    /// Chunks arrived of chunks planned, below 1 until the fetches end
    /// (reads beyond the plan, such as shard indexes, never show 100 %);
    /// `None` when no reader planned any.
    pub fraction: Option<f32>,
}

/// A fetch needs `chunks` more chunks.
pub fn plan(chunks: u64) {
    PLANNED.fetch_add(chunks, Ordering::Relaxed);
}

/// `chunks` chunks arrived (a missing one counts: it is settled).
pub fn arrived(chunks: u64) {
    ARRIVED.fetch_add(chunks, Ordering::Relaxed);
}

/// `n` network bytes arrived.
pub fn bytes(n: u64) {
    BYTES.fetch_add(n, Ordering::Relaxed);
}

pub fn snapshot() -> Snapshot {
    let planned = PLANNED.load(Ordering::Relaxed);
    let arrived = ARRIVED.load(Ordering::Relaxed);
    Snapshot {
        bytes: BYTES.load(Ordering::Relaxed),
        fraction: (planned > 0).then(|| (arrived as f32 / planned as f32).min(0.99)),
    }
}

/// Forgets everything counted: no fetch runs, or a new burst starts.
pub fn reset() {
    for counter in [&PLANNED, &ARRIVED, &BYTES] {
        counter.store(0, Ordering::Relaxed);
    }
}
