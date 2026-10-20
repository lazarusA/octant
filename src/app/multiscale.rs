//! Plotting multiscale variables: the UI stages level 0 (its sliders and
//! ranges are level-0 indices) and the plot reads one level, chosen from what
//! the plot type needs (`PlotController::resolution_demand`) or pinned.
//!
//! A level is a variable of its own, so the plotted selection is the staged
//! one moved onto that level's variable (`at_level`): requests, the cache,
//! coordinates, hover and axes then read the level as any variable.
//! `at_base` moves it back. Dimensions a level keeps whole (time, channel)
//! keep their indices, so playback steps are the same on every level.

use std::ops::Range;

use crate::app::layers::{LevelChoice, VariableSelection};
use crate::app::{DimConfig, OctantApp};
use crate::data::multiscale::{fit_level, pinned_level};
use crate::data::{DatasetMetadata, VariableInfo};

/// The index of variable `path` in `meta`.
pub(super) fn variable_index(meta: &DatasetMetadata, path: &str) -> Option<usize> {
    meta.variables.iter().position(|v| v.name == path)
}

/// The index of level 0 of the stack holding variable `idx` (`idx` itself
/// when it is in none).
pub(crate) fn stack_base_index(meta: &DatasetMetadata, idx: usize) -> usize {
    meta.variables
        .get(idx)
        .and_then(|v| v.multiscale.as_ref())
        .and_then(|r| variable_index(meta, &r.set.base().path))
        .unwrap_or(idx)
}

/// Whether variables `a` and `b` of `meta` are the same data (equal, or
/// levels of one stack).
pub(crate) fn same_variable(meta: &DatasetMetadata, a: usize, b: usize) -> bool {
    a == b || stack_base_index(meta, a) == stack_base_index(meta, b)
}

/// Variable `idx` of `meta` on the grid of the `base` variable: the level
/// of its stack with the same factor, offset and length as the base's level
/// along every dimension named like one of the base's, else its stack's
/// level 0 (overlays follow the level the base layer reads).
pub(crate) fn level_matching(
    base: Option<&VariableInfo>,
    meta: &DatasetMetadata,
    idx: usize,
) -> usize {
    let Some(set) = meta.variables.get(idx).and_then(|v| v.multiscale.as_ref()) else {
        return idx;
    };
    let fallback = stack_base_index(meta, idx);
    let Some(theirs) = base.and_then(|v| v.multiscale.as_ref()) else {
        return fallback;
    };
    let Some(target) = theirs.info() else {
        return fallback;
    };
    let same_grid = |level: &crate::data::multiscale::ScaleLevel| {
        set.set.dims.iter().enumerate().all(|(d, name)| {
            theirs.set.dim_index(name).is_none_or(|b| {
                (level.factor[d] - target.factor[b]).abs() < 1e-6
                    && (level.offset[d] - target.offset[b]).abs() < 1e-6
                    && level.shape[d] == target.shape[b]
            })
        })
    };
    set.set
        .levels
        .iter()
        .find(|l| same_grid(l))
        .and_then(|l| variable_index(meta, &l.path))
        .unwrap_or(fallback)
}

/// `selection`'s window as half-open ranges, one per dimension.
fn window(selection: &VariableSelection, shape: &[u64]) -> Vec<Range<u64>> {
    (0..shape.len())
        .map(|d| match selection.dim_ranges.get(d) {
            Some(&(s, e)) => s.min(e) as u64..s.max(e) as u64 + 1,
            None => 0..shape[d],
        })
        .collect()
}

/// `selection` (of a stack's level 0) moved onto `level`: that level's
/// variable, with every range, index and role window mapped onto its cells.
/// `None` when the variable is not level 0 of a stack or the level's
/// variable is missing.
pub(crate) fn at_level(selection: &VariableSelection, level: usize) -> Option<VariableSelection> {
    let var = selection.variable_info()?;
    let set = var.stack_base()?.set.clone();
    let meta = selection.metadata.as_ref()?;
    let idx = variable_index(meta, &set.level(level)?.path)?;
    let mut moved = selection.clone();
    moved.variable_idx = idx;
    let range = |d: usize, r: (usize, usize)| level_range(&set, level, d, r);
    let index = |d: usize, i: usize| {
        set.level_index(level, d, i as u64)
            .map_or(i, |i| i as usize)
    };
    map_dims(&mut moved, range, index);
    Some(moved)
}

/// Inclusive level-0 range `(s, e)` of dimension `d` on `level`'s cells.
pub(super) fn level_range(
    set: &crate::data::multiscale::Multiscale,
    level: usize,
    d: usize,
    (s, e): (usize, usize),
) -> (usize, usize) {
    let r = set.to_level(level, d, s.min(e) as u64..s.max(e) as u64 + 1);
    r.map_or((s, e), |r| {
        (r.start as usize, r.end.saturating_sub(1) as usize)
    })
}

/// `selection` (of a level other than 0) moved back onto its stack's level
/// 0, its ranges covering the level-0 cells under the level's. `None` when
/// the variable is no such level.
pub(crate) fn at_base(selection: &VariableSelection) -> Option<VariableSelection> {
    let var = selection.variable_info()?;
    let r = var.multiscale.as_ref().filter(|r| !r.is_base())?;
    let (set, level) = (r.set.clone(), r.level);
    let meta = selection.metadata.as_ref()?;
    let mut moved = selection.clone();
    moved.variable_idx = variable_index(meta, &set.base().path)?;
    let range = |d: usize, (s, e): (usize, usize)| {
        let r = set.to_base(level, d, s.min(e) as u64..s.max(e) as u64 + 1);
        r.map_or((s, e), |r| {
            (r.start as usize, r.end.saturating_sub(1) as usize)
        })
    };
    let index = |d: usize, i: usize| set.base_index(level, d, i as u64).map_or(i, |i| i as usize);
    map_dims(&mut moved, range, index);
    Some(moved)
}

/// `selection` (of a stack's level 0) moved onto the level whose variable
/// is `path`; `None` when that is level 0 or no level of the stack.
pub(crate) fn at_level_named(
    selection: &VariableSelection,
    path: &str,
) -> Option<VariableSelection> {
    let set = &selection.variable_info()?.stack_base()?.set;
    let path = path.trim_matches('/');
    let level = set
        .levels
        .iter()
        .position(|l| l.path.trim_matches('/') == path)?;
    (level > 0).then(|| at_level(selection, level)).flatten()
}

/// `selection`, of any level of a stack (or of no stack, as level 0),
/// moved onto its `level`.
pub(crate) fn to_level(selection: &VariableSelection, level: usize) -> Option<VariableSelection> {
    match (at_base(selection), level) {
        (Some(base), 0) => Some(base),
        (Some(base), level) => at_level(&base, level),
        (None, 0) => Some(selection.clone()),
        (None, level) => at_level(selection, level),
    }
}

/// Maps every range and index of `selection` (and of its dimension
/// configs) with `range` and `index`; a collapsed range stays one index.
fn map_dims(
    selection: &mut VariableSelection,
    range: impl Fn(usize, (usize, usize)) -> (usize, usize),
    index: impl Fn(usize, usize) -> usize,
) {
    let map_range = |d: usize, r: (usize, usize)| {
        if r.0 == r.1 {
            let i = index(d, r.0);
            (i, i)
        } else {
            range(d, r)
        }
    };
    for (d, r) in selection.dim_ranges.iter_mut().enumerate() {
        *r = map_range(d, *r);
    }
    for (d, i) in selection.dim_indices.iter_mut().enumerate() {
        *i = index(d, *i);
    }
    for (d, c) in selection.dim_config.iter_mut().enumerate() {
        *c = DimConfig {
            range: map_range(d, c.range),
            index: index(d, c.index),
            ..*c
        };
    }
}

impl OctantApp {
    /// The level `selection` reads when its variable is level 0 of a stack:
    /// the one its plot type asks for on the current canvas, or the pinned
    /// one within the memory budget.
    pub(crate) fn selection_level(&self, selection: &VariableSelection) -> Option<usize> {
        let var = selection.variable_info()?;
        let set = &var.stack_base()?.set;
        let demand = crate::app::controllers::controller_for(selection.plot_type)
            .resolution_demand(var, &selection.dim_config, self.layout.canvas_px);
        let window = window(selection, &var.shape);
        Some(match selection.level {
            LevelChoice::Auto => fit_level(set, &window, &demand),
            LevelChoice::Pinned(level) => pinned_level(set, level, &window, &demand),
        })
    }

    /// The index of the variable `selection` reads (its level's, for a
    /// stack's level 0), without moving the selection there.
    pub(crate) fn level_variable_idx(&self, selection: &VariableSelection) -> usize {
        let level = self.selection_level(selection).filter(|&l| l > 0);
        let idx = level.and_then(|l| {
            let set = &selection.variable_info()?.stack_base()?.set;
            variable_index(selection.metadata.as_ref()?, &set.level(l)?.path)
        });
        idx.unwrap_or(selection.variable_idx)
    }

    /// `selection` moved onto the level it reads; `None` when it reads its
    /// own variable (not a stack's level 0, or level 0 itself).
    pub(crate) fn resolve_level(&self, selection: &VariableSelection) -> Option<VariableSelection> {
        match self.selection_level(selection)? {
            0 => None,
            level => at_level(selection, level),
        }
    }
}
