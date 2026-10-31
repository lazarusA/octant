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

/// The chunk cache size of `array`: the chunks of one window as large as a
/// plot holds, else eight chunks, within 64-512 MB, at most all its chunks.
/// A window reading one band (or step) decodes chunks holding the others
/// too, so with them cached the next band reads nothing again.
pub(super) fn chunk_cache_bytes(array: &ZarrArrayHandle) -> u64 {
    let elem_size = match array.data_type().size() {
        zarrs::array::DataTypeSize::Fixed(s) => (s as u64).max(1),
        zarrs::array::DataTypeSize::Variable => 4,
    };
    let shape = array.shape();
    let Ok(chunk) = array.chunk_shape(&vec![0; shape.len()]) else {
        return 64 * 1024 * 1024;
    };
    let chunk: Vec<u64> = chunk.iter().map(|c| c.get()).collect();
    // Every chunk decodes whole, so the array's size rounds up to chunks.
    let (mut one_chunk, mut whole) = (Some(elem_size), Some(elem_size));
    for (&d, &c) in shape.iter().zip(&chunk) {
        one_chunk = one_chunk.and_then(|v| v.checked_mul(c));
        whole = whole.and_then(|v| v.checked_mul(d.div_ceil(c).checked_mul(c)?));
    }
    let (one_chunk, whole) = (one_chunk.unwrap_or(u64::MAX), whole.unwrap_or(u64::MAX));
    // The browser keeps the downloaded chunks themselves (`PartialStore`),
    // within its own budget, so it re-decodes rather than re-downloads.
    let window = match cfg!(target_arch = "wasm32") {
        true => 0,
        false => {
            let names = crate::utils::resolve_array_dimension_names(array);
            window_chunks(shape, &chunk, &names).saturating_mul(one_chunk)
        }
    };
    one_chunk
        .saturating_mul(8)
        .max(window)
        .clamp(64 * 1024 * 1024, 512 * 1024 * 1024)
        .min(whole)
}

/// The chunks a window of `MAX_GPU_STORAGE_BUFFER_ELEMENTS` cells (the
/// largest a 2D plot reads) touches over the array's x and y dimensions (by
/// `names`, else its last two): a square unaligned with the chunk grid,
/// longer along the long dimension when the short one is narrower. Other
/// dimensions (time, band) count one chunk, as a plot reads one step.
fn window_chunks(shape: &[u64], chunk: &[u64], names: &[String]) -> u64 {
    use crate::data::coordinates::naming::{is_spatial_x_name, is_spatial_y_name};
    let cells = crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS as u64;
    let named = |is: fn(&str) -> bool| names.iter().position(|n| is(n));
    let (a, b) = match (named(is_spatial_y_name), named(is_spatial_x_name)) {
        (Some(y), Some(x)) if y != x && x.max(y) < shape.len() => (y, x),
        _ => match shape.len() {
            0 => return 1,
            1 => (0, 0),
            n => (n - 2, n - 1),
        },
    };
    let (short, long) = if shape[a] <= shape[b] { (a, b) } else { (b, a) };
    let short_cells = shape[short].min(cells.isqrt()).max(1);
    let long_cells = shape[long].min(cells / short_cells);
    let crossed = |d: usize, n: u64| {
        let c = chunk[d].max(1);
        (n.div_ceil(c) + 1).min(shape[d].div_ceil(c))
    };
    match short == long {
        true => crossed(long, long_cells),
        false => crossed(short, short_cells) * crossed(long, long_cells),
    }
}

#[cfg(test)]
mod tests {
    use super::window_chunks;

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_window_holds_every_chunk_a_plot_reads() {
        let rgb = names(&["lat", "lon", "band"]);
        // TESSERA dclimate's coarsest RGB level, read whole: all 7 x 14 chunks.
        assert_eq!(window_chunks(&[3515, 7031, 4], &[512, 512, 4], &rgb), 98);
        // Its finest level: a 5792-cell square crosses at most 13 chunks a side.
        let finest = [1_800_000, 3_600_000, 4];
        assert_eq!(window_chunks(&finest, &[512, 512, 4], &rgb), 169);
        // Unnamed dimensions: the last two.
        let unnamed = names(&["dim_0", "dim_1", "dim_2"]);
        assert_eq!(
            window_chunks(&[4, 3515, 7031], &[4, 512, 512], &unnamed),
            98
        );
    }

    #[test]
    fn a_long_time_axis_is_no_window() {
        // Hourly maps, one step per chunk: a map reads one chunk, however
        // many steps the series has.
        let era5 = names(&["time", "latitude", "longitude"]);
        assert_eq!(window_chunks(&[744, 721, 1440], &[1, 721, 1440], &era5), 1);
    }
}
