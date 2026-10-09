//! Opacity curve rows of the colormap atlas: one per curve key (a layer's
//! id), in key order, after the colormap rows (see [`super::registry`]).

use super::lut::{Lut, sample_alpha};
use super::registry::{bump_generation, colormap_rows};
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Opacity curves baked by `alpha::bake`, by curve key (one per layer) in key
/// order, uploaded as the last atlas rows.
static ALPHA: RwLock<Vec<(u32, Box<Lut>)>> = RwLock::new(Vec::new());
/// Number of curves, kept in sync under the write lock.
static ALPHA_LEN: AtomicUsize = AtomicUsize::new(0);

pub(super) fn curves() -> RwLockReadGuard<'static, Vec<(u32, Box<Lut>)>> {
    ALPHA.read().unwrap_or_else(|p| p.into_inner())
}

/// Number of curve rows.
pub(super) fn count() -> usize {
    ALPHA_LEN.load(Ordering::Acquire)
}

/// Sets or clears the opacity curve of `key` (a layer's).
pub fn set_alpha_curve(key: u32, lut: Option<Box<Lut>>) {
    let mut curves = ALPHA.write().unwrap_or_else(|p| p.into_inner());
    let at = curves.binary_search_by_key(&key, |(k, _)| *k);
    match (at, lut) {
        (Ok(i), Some(lut)) => {
            if let Some(slot) = curves.get_mut(i) {
                slot.1 = lut;
            }
        }
        (Err(i), Some(lut)) => curves.insert(i, (key, lut)),
        (Ok(i), None) => {
            curves.remove(i);
        }
        (Err(_), None) => return,
    }
    ALPHA_LEN.store(curves.len(), Ordering::Release);
    bump_generation();
}

/// Atlas row of `key`'s opacity curve, if it has one.
pub fn alpha_row(key: u32) -> Option<u32> {
    let i = curves().binary_search_by_key(&key, |(k, _)| *k).ok()?;
    u32::try_from(colormap_rows() + i).ok()
}

/// `key`'s opacity curve at data position `t`; 1 without a curve.
pub fn curve_alpha(key: u32, t: f32) -> f32 {
    let curves = curves();
    let at = curves.binary_search_by_key(&key, |(k, _)| *k);
    at.ok()
        .and_then(|i| curves.get(i))
        .map_or(1.0, |(_, lut)| sample_alpha(lut, t))
}

/// The opacity curve in atlas row `row` at data position `t`; 1 when the
/// row holds no curve.
pub fn row_curve_alpha(row: u32, t: f32) -> f32 {
    let curves = curves();
    (row as usize)
        .checked_sub(colormap_rows())
        .and_then(|i| curves.get(i))
        .map_or(1.0, |(_, lut)| sample_alpha(lut, t))
}
