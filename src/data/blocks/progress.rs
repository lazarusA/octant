//! Download progress of the running block fetches, shared by every reader:
//! each fetch adds the chunks its window needs (`plan`) and each chunk as it
//! arrives (`arrived`), with the network bytes it took (`bytes`). Counted
//! from the first fetch until none runs (`BlockPrefetcher`, `reset`), so the
//! status bar shows one figure for every fetch in flight.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

static PLANNED: AtomicU64 = AtomicU64::new(0);
static ARRIVED: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
/// Key prefixes of the arrays whose blocks are fetched (desktop): only
/// their reads count as arrived chunks, not coordinate or metadata reads.
#[cfg(not(target_arch = "wasm32"))]
static ARRAYS: Mutex<Vec<String>> = Mutex::new(Vec::new());

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

/// A fetch of the array whose keys start with `prefix` needs `chunks` more
/// chunks (desktop, `zarr/counting.rs`).
#[cfg(not(target_arch = "wasm32"))]
pub fn plan_array(prefix: &str, chunks: u64) {
    plan(chunks);
    let mut arrays = ARRAYS.lock().unwrap_or_else(|p| p.into_inner());
    if !arrays.iter().any(|a| a == prefix) {
        arrays.push(prefix.to_string());
    }
}

/// Whether `key` belongs to an array a fetch planned (`plan_array`).
#[cfg(not(target_arch = "wasm32"))]
pub fn is_planned(key: &str) -> bool {
    let arrays = ARRAYS.lock().unwrap_or_else(|p| p.into_inner());
    arrays.iter().any(|a| key.starts_with(a.as_str()))
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
    #[cfg(not(target_arch = "wasm32"))]
    ARRAYS.lock().unwrap_or_else(|p| p.into_inner()).clear();
}
