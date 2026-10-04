//! Process-wide colormap registry: built-in catalog followed by user-defined maps.
//!
//! A colormap id is a row index: `0..builtin_len` are bundled maps, then custom
//! maps in insertion order. The id is also the row of the GPU atlas. After those
//! [`len`] rows come the smooth twins of short categorical palettes (see
//! [`smooth_variant`]), for [`rows`] rows in total. [`generation`] changes
//! whenever rows change so GPU and UI caches can refresh.

use super::catalog::{ColormapEntry, builtin};
use super::lut::{Lut, sample_lut};
use egui::Color32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, RwLock, RwLockReadGuard};

/// Uniform `colormap` value for direct RGB composite (truecolor) rendering.
pub const COLORMAP_RGB_COMPOSITE: u32 = u32::MAX;

/// Key of the colormap selected on startup.
pub const DEFAULT_COLORMAP_KEY: &str = "matplotlib:viridis";

static CUSTOM: LazyLock<RwLock<Vec<ColormapEntry>>> = LazyLock::new(|| RwLock::new(Vec::new()));
static GENERATION: AtomicU64 = AtomicU64::new(1);

fn custom() -> RwLockReadGuard<'static, Vec<ColormapEntry>> {
    CUSTOM.read().unwrap_or_else(|p| p.into_inner())
}

/// Bumped whenever the set of rows changes.
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Acquire)
}

pub fn builtin_len() -> usize {
    builtin().maps.len()
}

/// Total number of colormaps (rows).
pub fn len() -> usize {
    builtin_len() + custom().len()
}

/// Runs `f` on the entry with this id.
pub fn with_entry<R>(id: u32, f: impl FnOnce(&ColormapEntry) -> R) -> Option<R> {
    let idx = id as usize;
    let maps = &builtin().maps;
    if let Some(entry) = maps.get(idx) {
        return Some(f(entry));
    }
    custom().get(idx - maps.len()).map(f)
}

/// Atlas rows: every colormap followed by the smooth twins.
pub fn rows() -> usize {
    let custom = custom();
    let entries = builtin().maps.iter().chain(custom.iter());
    builtin_len() + custom.len() + entries.filter(|e| e.smooth.is_some()).count()
}

/// Visits every atlas row in order: colormaps first, then smooth twins.
pub fn for_each_lut(mut f: impl FnMut(u32, &Lut)) {
    let custom = custom();
    let entries = || builtin().maps.iter().chain(custom.iter());
    let luts = entries()
        .map(|e| &*e.lut)
        .chain(entries().filter_map(|e| e.smooth.as_deref()));
    for (row, lut) in luts.enumerate() {
        f(u32::try_from(row).unwrap_or(u32::MAX), lut);
    }
}

/// Atlas row of the smooth twin of colormap `id`, if it has one.
pub fn smooth_variant(id: u32) -> Option<u32> {
    let custom = custom();
    let entries = || builtin().maps.iter().chain(custom.iter());
    let idx = id as usize;
    entries().nth(idx)?.smooth.as_ref()?;
    let before = entries().take(idx).filter(|e| e.smooth.is_some()).count();
    u32::try_from(builtin_len() + custom.len() + before).ok()
}

/// Runs `f` on the LUT of an atlas row (a colormap or a smooth twin).
fn with_row_lut<R>(row: u32, f: impl FnOnce(&Lut) -> R) -> Option<R> {
    let custom = custom();
    let entries = || builtin().maps.iter().chain(custom.iter());
    let row = row as usize;
    let base = builtin_len() + custom.len();
    if row < base {
        return entries().nth(row).map(|e| f(&e.lut));
    }
    entries()
        .filter_map(|e| e.smooth.as_deref())
        .nth(row - base)
        .map(f)
}

/// Finds a colormap id by its stable key (`"family:name"`).
pub fn find(key: &str) -> Option<u32> {
    let maps = &builtin().maps;
    if let Some(i) = maps.iter().position(|e| e.key == key) {
        return u32::try_from(i).ok();
    }
    let i = custom().iter().position(|e| e.key == key)?;
    u32::try_from(maps.len() + i).ok()
}

/// Id of the default colormap.
pub fn default_id() -> u32 {
    find(DEFAULT_COLORMAP_KEY).unwrap_or(0)
}

/// Stable key for an id, falling back to the default key for unknown ids.
pub fn key_of(id: u32) -> String {
    with_entry(id, |e| e.key.clone()).unwrap_or_else(|| DEFAULT_COLORMAP_KEY.to_string())
}

/// Samples atlas row `id` (a colormap or a smooth twin) at `t` in [0, 1];
/// unknown rows use the first map.
pub fn sample(id: u32, t: f32) -> Color32 {
    with_row_lut(id, |lut| sample_lut(lut, t))
        .or_else(|| builtin().maps.first().map(|e| sample_lut(&e.lut, t)))
        .unwrap_or(Color32::BLACK)
}

/// Adds a custom map, or replaces the one with the same key. Returns its id.
pub fn upsert_custom(entry: ColormapEntry) -> u32 {
    let mut maps = CUSTOM.write().unwrap_or_else(|p| p.into_inner());
    let idx = match maps.iter().position(|e| e.key == entry.key) {
        Some(i) => {
            maps[i] = entry;
            i
        }
        None => {
            maps.push(entry);
            maps.len() - 1
        }
    };
    GENERATION.fetch_add(1, Ordering::AcqRel);
    u32::try_from(builtin_len() + idx).unwrap_or(0)
}

/// Removes a custom map by key. Ids of later custom maps shift down by one.
pub fn remove_custom(key: &str) -> bool {
    let mut maps = CUSTOM.write().unwrap_or_else(|p| p.into_inner());
    let before = maps.len();
    maps.retain(|e| e.key != key);
    let removed = maps.len() != before;
    if removed {
        GENERATION.fetch_add(1, Ordering::AcqRel);
    }
    removed
}
