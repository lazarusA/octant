//! The arrays a Zarr store has open, each with its chunk cache: opened once
//! however many block fetches ask at the same time, the least recently used
//! closed once their caches pass the budget.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::block::ArrayCachedState;
use super::storage::ZarrArrayHandle;

/// Decoded chunk bytes the open arrays of one store may hold together: the
/// least recently used arrays are closed beyond it (an OME-Zarr image opens
/// one array per level, each with its own chunk cache).
#[cfg(not(target_arch = "wasm32"))]
const OPEN_ARRAYS_BUDGET: u64 = 1024 * 1024 * 1024;
#[cfg(target_arch = "wasm32")]
const OPEN_ARRAYS_BUDGET: u64 = 256 * 1024 * 1024;

/// Open arrays with the size of their chunk caches, least recently used first.
type Listed = Vec<(String, u64, Arc<ArrayCachedState>)>;

/// A store's open arrays.
#[derive(Default)]
pub(super) struct OpenArrays {
    listed: Mutex<Listed>,
    /// One lock per array being opened, so concurrent block fetches (prefetch
    /// workers) share one array and its chunk cache instead of each opening
    /// it, while different arrays open in parallel.
    opening: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl OpenArrays {
    /// The open array `name`, else opened by `open` (returning the array and
    /// its chunk cache size), once at a time per array.
    pub(super) fn get_or_open<E>(
        &self,
        name: &str,
        open: impl Fn(&str) -> Result<(Arc<ArrayCachedState>, u64), E>,
    ) -> Result<Arc<ArrayCachedState>, E> {
        loop {
            if let Some(found) = self.get(name) {
                return Ok(found);
            }
            let slot = {
                let mut opening = self.opening.lock().unwrap_or_else(|p| p.into_inner());
                opening.entry(name.to_string()).or_default().clone()
            };
            let _opening = slot.lock().unwrap_or_else(|p| p.into_inner());
            // A failed open released this slot: start over under the current one.
            if !self.holds(name, &slot) {
                continue;
            }
            if let Some(found) = self.get(name) {
                return Ok(found);
            }
            let opened = open(name).map(|(cached, cache_bytes)| {
                self.list(name, cache_bytes, &cached);
                cached
            });
            // Released once the array is listed: later callers find it there.
            let mut opening = self.opening.lock().unwrap_or_else(|p| p.into_inner());
            if opening.get(name).is_some_and(|s| Arc::ptr_eq(s, &slot)) {
                opening.remove(name);
            }
            return opened;
        }
    }

    /// An array already open, marked most recently used.
    fn get(&self, name: &str) -> Option<Arc<ArrayCachedState>> {
        let mut listed = self.listed.lock().unwrap_or_else(|p| p.into_inner());
        let i = listed.iter().position(|(n, _, _)| n == name)?;
        let entry = listed.remove(i);
        let cached = entry.2.clone();
        listed.push(entry);
        Some(cached)
    }

    /// Whether `slot` is still the one array `name` opens under.
    fn holds(&self, name: &str, slot: &Arc<Mutex<()>>) -> bool {
        let opening = self.opening.lock().unwrap_or_else(|p| p.into_inner());
        opening.get(name).is_some_and(|s| Arc::ptr_eq(s, slot))
    }

    /// Lists `cached` open, closing the least recently used past the budget.
    fn list(&self, name: &str, cache_bytes: u64, cached: &Arc<ArrayCachedState>) {
        let mut listed = self.listed.lock().unwrap_or_else(|p| p.into_inner());
        listed.push((name.to_string(), cache_bytes, cached.clone()));
        while listed.len() > 1 && listed.iter().map(|(_, b, _)| b).sum::<u64>() > OPEN_ARRAYS_BUDGET
        {
            listed.remove(0);
        }
    }
}

/// The chunk cache size of `array`: eight chunks within 64-512 MB, at most all its chunks.
pub(super) fn chunk_cache_bytes(array: &ZarrArrayHandle) -> u64 {
    let elem_size = match array.data_type().size() {
        zarrs::array::DataTypeSize::Fixed(s) => (s as u64).max(1),
        zarrs::array::DataTypeSize::Variable => 4,
    };
    let shape = array.shape();
    let Ok(chunk) = array.chunk_shape(&vec![0; shape.len()]) else {
        return 64 * 1024 * 1024;
    };
    // Every chunk decodes whole, so the array's size rounds up to chunks.
    let (mut one_chunk, mut whole) = (Some(elem_size), Some(elem_size));
    for (&d, c) in shape.iter().zip(chunk.iter()) {
        let c = c.get();
        one_chunk = one_chunk.and_then(|v| v.checked_mul(c));
        whole = whole.and_then(|v| v.checked_mul(d.div_ceil(c).checked_mul(c)?));
    }
    let (one_chunk, whole) = (one_chunk.unwrap_or(u64::MAX), whole.unwrap_or(u64::MAX));
    one_chunk
        .saturating_mul(8)
        .clamp(64 * 1024 * 1024, 512 * 1024 * 1024)
        .min(whole)
}
