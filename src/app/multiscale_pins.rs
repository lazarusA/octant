//! The level-0 indices a selection moved onto a coarser level came from
//! (`multiscale::at_level`). A coarse cell covers several level-0 indices, so
//! `at_base` gives back the remembered ones while the level's indices still
//! map onto them, rather than the index under the coarse cell's center: a
//! collapsed z at 13 read at a level of factor 8 stays 13, not 12.

use std::sync::Arc;

use crate::app::layers::VariableSelection;
use crate::data::multiscale::Multiscale;

/// A level-0 selection's indices and ranges, kept on its copies at coarser
/// levels (`VariableSelection::level0`).
#[derive(Debug, Clone)]
pub struct Level0Indices {
    dim_indices: Vec<usize>,
    dim_ranges: Vec<(usize, usize)>,
    /// Each dimension config's index and range.
    config: Vec<(usize, (usize, usize))>,
}

impl Level0Indices {
    pub(crate) fn of(selection: &VariableSelection) -> Arc<Self> {
        Arc::new(Self {
            dim_indices: selection.dim_indices.clone(),
            dim_ranges: selection.dim_ranges.clone(),
            config: selection
                .dim_config
                .iter()
                .map(|c| (c.index, c.range))
                .collect(),
        })
    }
}

/// Puts back into `moved` (`from`, a selection of `level`, moved onto level
/// 0) the level-0 indices and collapsed ranges `from` was made from, where
/// they still map onto `from`'s.
pub(crate) fn restore(
    moved: &mut VariableSelection,
    from: &VariableSelection,
    set: &Multiscale,
    level: usize,
) {
    let Some(was) = from.level0.as_deref() else {
        return;
    };
    let maps = |d: usize, w: usize, n: usize| set.level_index(level, d, w as u64) == Some(n as u64);
    let index = |d: usize, w: Option<usize>, n: Option<usize>| match (w, n) {
        (Some(w), Some(n)) if maps(d, w, n) => Some(w),
        _ => None,
    };
    let range = |d: usize, w: Option<(usize, usize)>, n: Option<(usize, usize)>| match (w, n) {
        (Some((a, b)), Some((c, e))) if a == b && c == e && maps(d, a, c) => Some((a, a)),
        _ => None,
    };
    for (d, i) in moved.dim_indices.iter_mut().enumerate() {
        let now = from.dim_indices.get(d).copied();
        if let Some(w) = index(d, was.dim_indices.get(d).copied(), now) {
            *i = w;
        }
    }
    for (d, r) in moved.dim_ranges.iter_mut().enumerate() {
        let now = from.dim_ranges.get(d).copied();
        if let Some(w) = range(d, was.dim_ranges.get(d).copied(), now) {
            *r = w;
        }
    }
    for (d, c) in moved.dim_config.iter_mut().enumerate() {
        let (w, n) = (was.config.get(d), from.dim_config.get(d));
        if let Some(i) = index(d, w.map(|w| w.0), n.map(|n| n.index)) {
            c.index = i;
        }
        if let Some(r) = range(d, w.map(|w| w.1), n.map(|n| n.range)) {
            c.range = r;
        }
    }
}
