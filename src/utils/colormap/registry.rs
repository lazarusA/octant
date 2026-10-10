//! Process-wide colormap registry: built-in catalog followed by user-defined maps.
//!
//! A colormap id is a row index: `0..builtin_len` are bundled maps, then custom
//! maps in insertion order. The id is also the row of the GPU atlas. After those
//! [`len`] rows come the smooth twins of short categorical palettes, built-in
//! first then custom (see [`smooth_variant`]), then one opacity curve row per
//! curve key that has one ([`set_alpha_curve`], in key order), for [`rows`]
//! rows in total.
//! [`generation`] changes whenever rows change so GPU and UI caches can refresh.

use super::catalog::{ColormapEntry, builtin};
use super::curve_rows;
pub use super::curve_rows::{alpha_row, curve_alpha, row_curve_alpha, set_alpha_curve};
use super::lut::{Lut, sample_lut};
use egui::Color32;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{LazyLock, RwLock, RwLockReadGuard};

/// Uniform `colormap` value for direct RGB composite (truecolor) rendering.
pub const COLORMAP_RGB_COMPOSITE: u32 = u32::MAX;

/// Key of the colormap selected on startup.
pub const DEFAULT_COLORMAP_KEY: &str = "matplotlib:viridis";

/// User-defined maps, plus the indexes of those with a smooth twin in row order.
#[derive(Default)]
struct Custom {
    maps: Vec<ColormapEntry>,
    twins: Vec<usize>,
}

impl Custom {
    /// Rebuilds `twins` and the lock-free counts after `maps` changed.
    fn reindex(&mut self) {
        self.twins = (0..self.maps.len())
            .filter(|&i| self.maps[i].smooth.is_some())
            .collect();
        CUSTOM_LEN.store(self.maps.len(), Ordering::Release);
        CUSTOM_TWINS.store(self.twins.len(), Ordering::Release);
        GENERATION.fetch_add(1, Ordering::AcqRel);
    }
}

static CUSTOM: LazyLock<RwLock<Custom>> = LazyLock::new(|| RwLock::new(Custom::default()));
static GENERATION: AtomicU64 = AtomicU64::new(1);
/// Number of custom maps, kept in sync under the write lock so built-in rows
/// and their twins resolve without taking the lock.
static CUSTOM_LEN: AtomicUsize = AtomicUsize::new(0);
/// Number of custom maps with a smooth twin, kept in sync the same way.
static CUSTOM_TWINS: AtomicUsize = AtomicUsize::new(0);
fn custom() -> RwLockReadGuard<'static, Custom> {
    CUSTOM.read().unwrap_or_else(|p| p.into_inner())
}

/// Bumped whenever the set of rows changes.
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Acquire)
}

fn builtin_len() -> usize {
    builtin().maps.len()
}

/// Number of colormaps (built-in + custom): the ids `0..len()`.
pub fn len() -> usize {
    builtin_len() + CUSTOM_LEN.load(Ordering::Acquire)
}

fn builtin_twins() -> usize {
    builtin().twins.len()
}

/// Runs `f` on the entry with this id.
pub fn with_entry<R>(id: u32, f: impl FnOnce(&ColormapEntry) -> R) -> Option<R> {
    let idx = id as usize;
    let maps = &builtin().maps;
    if let Some(entry) = maps.get(idx) {
        return Some(f(entry));
    }
    custom().maps.get(idx - maps.len()).map(f)
}

/// Colormap rows: every colormap, then the built-in smooth twins, then the
/// twins of custom maps.
pub(super) fn colormap_rows() -> usize {
    len() + builtin_twins() + CUSTOM_TWINS.load(Ordering::Acquire)
}

/// Atlas rows: the colormap rows, then one row per opacity curve.
pub fn rows() -> usize {
    colormap_rows() + curve_rows::count()
}

/// Marks the rows as changed, so GPU and UI caches refresh.
pub(super) fn bump_generation() {
    GENERATION.fetch_add(1, Ordering::AcqRel);
}

/// Visits every atlas row in order (see [`rows`]).
pub fn for_each_lut(mut f: impl FnMut(u32, &Lut)) {
    let custom = custom();
    let entries = || builtin().maps.iter().chain(custom.maps.iter());
    let luts = entries()
        .map(|e| &*e.lut)
        .chain(entries().filter_map(|e| e.smooth.as_deref()));
    let curves = curve_rows::curves();
    for (row, lut) in luts.chain(curves.iter().map(|(_, lut)| &**lut)).enumerate() {
        f(u32::try_from(row).unwrap_or(u32::MAX), lut);
    }
}

/// Atlas row of the smooth twin of colormap `id`, if it has one. O(1) for
/// built-in maps, O(log n) for custom maps.
pub fn smooth_variant(id: u32) -> Option<u32> {
    let idx = id as usize;
    let twin = if let Some(k) = builtin().twin_of.get(idx) {
        (*k)? as usize
    } else {
        let custom = custom();
        let local = idx - builtin_len();
        custom.maps.get(local)?.smooth.as_ref()?;
        builtin_twins() + custom.twins.partition_point(|&i| i < local)
    };
    u32::try_from(len() + twin).ok()
}

/// Runs `f` on the LUT of an atlas row (a colormap or a smooth twin). Built-in
/// rows and built-in twins resolve without locking.
fn with_row_lut<R>(row: u32, f: impl FnOnce(&Lut) -> R) -> Option<R> {
    let row = row as usize;
    if row < len() {
        return with_entry(row as u32, |e| f(&e.lut));
    }
    let k = row - len();
    if let Some(&map) = builtin().twins.get(k) {
        return builtin().maps.get(map as usize)?.smooth.as_deref().map(f);
    }
    let custom = custom();
    let map = *custom.twins.get(k - builtin_twins())?;
    custom.maps.get(map)?.smooth.as_deref().map(f)
}

/// Finds a colormap id by its stable key (`"family:name"`).
pub fn find(key: &str) -> Option<u32> {
    let maps = &builtin().maps;
    if let Some(i) = maps.iter().position(|e| e.key == key) {
        return u32::try_from(i).ok();
    }
    let i = custom().maps.iter().position(|e| e.key == key)?;
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

/// Whether atlas row `row` is a stepped (categorical) map, sampled with the
/// nearest texel on both CPU and GPU. Smooth twins are continuous.
pub fn is_stepped(row: u32) -> bool {
    (row as usize) < len() && with_entry(row, |e| e.kind.is_discrete()).unwrap_or(false)
}

/// Whether `row` is a colormap or smooth twin row (not the opacity curve).
pub fn is_row(row: u32) -> bool {
    (row as usize) < colormap_rows()
}

/// Samples atlas row `id` (a colormap or a smooth twin) at `t` in [0, 1];
/// unknown rows use the default colormap.
pub fn sample(id: u32, t: f32) -> Color32 {
    sample_row(id, t, is_stepped(id))
}

/// [`sample`] with the nearest-texel flag given by the caller (the plot
/// uniforms carry it), as WGSL `sample_colormap` does. Unknown rows sample the
/// default colormap with its own flag, as `ColorStyle::params` draws them.
pub fn sample_row(id: u32, t: f32, nearest: bool) -> Color32 {
    with_row_lut(id, |lut| sample_lut(lut, t, nearest))
        .or_else(|| {
            let default = default_id();
            with_row_lut(default, |lut| sample_lut(lut, t, is_stepped(default)))
        })
        .unwrap_or(Color32::BLACK)
}

/// Adds a custom map, or replaces the one with the same key. Returns its id.
pub fn upsert_custom(entry: ColormapEntry) -> u32 {
    let mut custom = CUSTOM.write().unwrap_or_else(|p| p.into_inner());
    let idx = match custom.maps.iter().position(|e| e.key == entry.key) {
        Some(i) => {
            custom.maps[i] = entry;
            i
        }
        None => {
            custom.maps.push(entry);
            custom.maps.len() - 1
        }
    };
    custom.reindex();
    u32::try_from(builtin_len() + idx).unwrap_or(0)
}

/// Removes a custom map by key. Ids of later custom maps shift down by one.
pub fn remove_custom(key: &str) -> bool {
    let mut custom = CUSTOM.write().unwrap_or_else(|p| p.into_inner());
    let before = custom.maps.len();
    custom.maps.retain(|e| e.key != key);
    let removed = custom.maps.len() != before;
    if removed {
        custom.reindex();
    }
    removed
}

/// Serializes tests that add or remove custom maps or depend on row ids: the
/// registry is process-wide and removing a custom map shifts later ids.
#[cfg(test)]
pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|p| p.into_inner())
}
