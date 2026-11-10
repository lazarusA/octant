//! The level-0 indices a selection moved onto a coarser level came from
//! (`multiscale::at_level`). A coarse cell covers several level-0 indices, so
//! `at_base` gives back the remembered ones while the level's indices still
//! map onto them, rather than the index under the coarse cell's center: a
//! collapsed z at 13 read at a level of factor 8 stays 13, not 12.

use std::sync::Arc;

use crate::app::layers::VariableSelection;
use crate::data::multiscale::Multiscale;

/// A level-0 selection's dimension indices and ranges, kept on its copies
/// at coarser levels (`VariableSelection::level0`).
#[derive(Debug, Clone)]
pub struct Level0Indices(Vec<(usize, (usize, usize))>);

impl Level0Indices {
    pub(crate) fn of(selection: &VariableSelection) -> Arc<Self> {
        let dims = selection.dim_config.iter().map(|c| (c.index, c.range));
        Arc::new(Self(dims.collect()))
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
    for (d, c) in moved.dim_config.iter_mut().enumerate() {
        let (Some(&(wi, wr)), Some(n)) = (was.0.get(d), from.dim_config.get(d)) else {
            continue;
        };
        if maps(d, wi, n.index) {
            c.index = wi;
        }
        let ((a, b), (s, e)) = (wr, n.range);
        if a == b && s == e && maps(d, a, s) {
            c.range = (a, a);
        }
    }
}
