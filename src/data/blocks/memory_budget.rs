//! The memory budget of downloaded data kept between loads, shared by every
//! backend that keeps any (the browser's Zarr and Icechunk chunks and shard
//! parts, Icechunk manifests). Past [`DOWNLOAD_BUDGET`], the least recently
//! used objects their holders let go are dropped (fetched again when asked
//! for), down to [`TRIM_TO`]. WebAssembly memory never shrinks, so the
//! budget is what caps its peak.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, Weak};

/// Bytes of downloaded data held by every holder together.
pub const DOWNLOAD_BUDGET: u64 = 256 * 1024 * 1024;
/// Held bytes once a trim is done, below the budget so that the next
/// downloads do not trim again at once.
const TRIM_TO: u64 = DOWNLOAD_BUDGET / 8 * 7;

/// Something holding downloaded objects the budget may drop.
pub trait Evictable: Send + Sync {
    /// Appends `(last use, holder, key)` of every object it may drop now,
    /// `holder` being its index in the trim.
    fn evictable(&self, holder: usize, out: &mut Vec<(u64, usize, String)>);
    /// Drops the object `key` unless it is in use again; the bytes freed.
    fn evict(&self, key: &str) -> u64;
}

/// The bytes held by the holders counting against one budget.
pub struct MemoryBudget {
    limit: u64,
    trim_to: u64,
    held: AtomicU64,
    /// Held bytes after a trim that could not get under the limit (what is
    /// left is in use); 0 when the last trim did. No trim runs again until
    /// something in use is let go or another `limit / 8` is held.
    stuck_at: AtomicU64,
    /// Orders uses: each use takes the next tick.
    clock: AtomicU64,
    holders: Mutex<Vec<Weak<dyn Evictable>>>,
}

impl MemoryBudget {
    /// A budget of `limit` bytes, trimmed down to `trim_to`.
    pub fn new(limit: u64, trim_to: u64) -> Arc<Self> {
        Arc::new(Self {
            limit,
            trim_to: trim_to.min(limit),
            held: AtomicU64::new(0),
            stuck_at: AtomicU64::new(0),
            clock: AtomicU64::new(0),
            holders: Mutex::new(Vec::new()),
        })
    }

    /// The budget every holder shares.
    pub fn shared() -> Arc<Self> {
        static SHARED: LazyLock<Arc<MemoryBudget>> =
            LazyLock::new(|| MemoryBudget::new(DOWNLOAD_BUDGET, TRIM_TO));
        SHARED.clone()
    }

    /// Bytes held now.
    pub fn held(&self) -> u64 {
        self.held.load(Ordering::Relaxed)
    }

    /// The next use's tick.
    pub fn tick(&self) -> u64 {
        self.clock.fetch_add(1, Ordering::Relaxed)
    }

    /// Lets the budget drop `holder`'s objects for as long as it lives.
    pub fn register<T: Evictable + 'static>(&self, holder: &Arc<T>) {
        let weak: Weak<dyn Evictable> = Arc::downgrade(holder) as Weak<dyn Evictable>;
        let mut holders = self.holders.lock().unwrap_or_else(|p| p.into_inner());
        holders.retain(|h| h.strong_count() > 0);
        holders.push(weak);
    }

    pub fn added(&self, bytes: u64) {
        self.held.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn released(&self, bytes: u64) {
        if bytes > 0 {
            let _ = self
                .held
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |h| {
                    Some(h.saturating_sub(bytes))
                });
        }
    }

    /// Once past the limit, drops the least recently used objects the
    /// holders let go, until at most `trim_to` bytes are held.
    pub fn trim(&self) {
        let held = self.held();
        let stuck_at = self.stuck_at.load(Ordering::Relaxed);
        if held <= self.limit || (stuck_at > 0 && held < stuck_at + self.limit / 8) {
            return;
        }
        let holders: Vec<Arc<dyn Evictable>> = {
            let mut holders = self.holders.lock().unwrap_or_else(|p| p.into_inner());
            holders.retain(|h| h.strong_count() > 0);
            holders.iter().filter_map(Weak::upgrade).collect()
        };
        let mut candidates = Vec::new();
        for (i, holder) in holders.iter().enumerate() {
            holder.evictable(i, &mut candidates);
        }
        candidates.sort_unstable_by_key(|c| c.0);
        for (_, i, key) in candidates {
            if self.held() <= self.trim_to {
                break;
            }
            self.released(holders[i].evict(&key));
        }
        let left = self.held();
        let stuck = if left > self.limit { left } else { 0 };
        self.stuck_at.store(stuck, Ordering::Relaxed);
    }

    /// Trims once objects in use were let go (a lease released), even if
    /// the last trim could not.
    pub fn trim_after_release(&self) {
        self.stuck_at.store(0, Ordering::Relaxed);
        self.trim();
    }
}
