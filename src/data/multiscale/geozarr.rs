//! GeoZarr pyramids (zarr-conventions `multiscales` v0.1) as multiscale stacks.
//!
//! A group's `multiscales` attribute is an object whose `layout` lists the
//! levels: each an `asset` (a child group, or an array) with an optional
//! `derived_from`, a `transform` whose `scale`/`translation` are relative to
//! that level, and its own `spatial:transform` affine
//! `[a, b, c, d, e, f]` (`x = a*col + b*row + c`, `y = d*col + e*row + f`).
//! OME-Zarr's `multiscales` is an array, so the two never meet.
//!
//! Factors and offsets of the spatial dimensions come from the levels' own
//! affines whenever both have one, since writers disagree on `translation`
//! (topozarr 0.1.9 wrote cell-centre shifts, 0.2.0 writes 0); other
//! dimensions use the chained `transform`, else the shape ratio.

use crate::data::metadata::VariableInfo;
use crate::data::ome::MetaReader;
use crate::data::ome::reader::join;

use super::geozarr_layout::{Entry, Layout};
use super::types::{MultiscaleKind, ScaleLevel};

/// Links the levels of every GeoZarr pyramid among `variables`. Groups are
/// looked for at each variable's group and that group's parent (a level is
/// a child group or an array of the pyramid's group); attributes the reader
/// cannot answer are skipped.
pub fn link_pyramids(reader: &MetaReader<'_>, variables: &mut [VariableInfo]) {
    let mut groups: Vec<String> = Vec::new();
    for v in variables.iter() {
        let group = v.group_path().unwrap_or("").to_string();
        let parent = group.rsplit_once('/').map_or("", |(p, _)| p).to_string();
        for g in [group, parent] {
            if !groups.contains(&g) {
                groups.push(g);
            }
        }
    }
    for group in groups {
        let Ok(Some(attrs)) = reader.group_attributes(&group) else {
            continue;
        };
        if let Some(layout) = Layout::parse(&attrs) {
            link_layout(&group, &layout, variables);
        }
    }
}

/// Links one stack per array name found at the base level of `layout`.
pub fn link_layout(group: &str, layout: &Layout, variables: &mut [VariableInfo]) {
    let Some(base) = layout.base() else {
        return;
    };
    let base_path = join(group, &base.asset);
    let names: Vec<String> = variables
        .iter()
        .filter_map(|v| relative_name(&v.name, &base_path))
        .map(str::to_string)
        .collect();
    for name in names {
        let levels = stack_levels(group, layout, base, &name, variables);
        let Some(first) = levels.first() else {
            continue;
        };
        let Some(v) = variables.iter().find(|v| v.name == first.path) else {
            continue;
        };
        let dims = v.dimension_names.clone();
        // An array asset takes the pyramid group's name (else the asset's).
        let label = match (name.is_empty(), group.is_empty()) {
            (false, _) => name.as_str(),
            (true, false) => leaf(group),
            (true, true) => base.asset.as_str(),
        };
        super::try_link(
            label.to_string(),
            dims,
            levels,
            MultiscaleKind::GeoZarr,
            variables,
        );
    }
}

/// `name` within level `asset_path`: `""` when the asset is the array
/// itself, the array's name when it is a direct child of the asset group.
fn relative_name<'a>(name: &'a str, asset_path: &str) -> Option<&'a str> {
    let name = name.trim_matches('/');
    if name == asset_path {
        return Some("");
    }
    let rest = if asset_path.is_empty() {
        name
    } else {
        name.strip_prefix(asset_path)?.strip_prefix('/')?
    };
    (!rest.contains('/')).then_some(rest)
}

fn leaf(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The variable named `name` at every level holding one of the base's rank,
/// finest first.
fn stack_levels(
    group: &str,
    layout: &Layout,
    base: &Entry,
    name: &str,
    variables: &[VariableInfo],
) -> Vec<ScaleLevel> {
    let find = |entry: &Entry| {
        let path = join(&join(group, &entry.asset), name);
        variables.iter().find(|v| v.name.trim_matches('/') == path)
    };
    let Some(base_var) = find(base) else {
        return Vec::new();
    };
    let mut levels: Vec<ScaleLevel> = layout
        .entries
        .iter()
        .filter_map(|entry| {
            let v = find(entry).filter(|v| v.shape.len() == base_var.shape.len())?;
            let (factor, offset) = placement(layout, base, base_var, entry, v);
            Some(ScaleLevel {
                path: v.name.clone(),
                shape: v.shape.clone(),
                factor,
                offset,
            })
        })
        .collect();
    levels.sort_by(|a, b| volume(&a.factor).total_cmp(&volume(&b.factor)));
    levels
}

fn volume(factor: &[f64]) -> f64 {
    factor.iter().product()
}

/// Factor and offset per dimension of `v` (at `entry`) on `base_var`'s grid.
fn placement(
    layout: &Layout,
    base: &Entry,
    base_var: &VariableInfo,
    entry: &Entry,
    v: &VariableInfo,
) -> (Vec<f64>, Vec<f64>) {
    let rank = v.shape.len();
    let chained = layout.chain(entry, rank);
    let (x, y) = layout.spatial_dims(entry, &v.dimension_names);
    let affine = base.affine.zip(entry.affine);
    (0..rank)
        .map(|d| {
            let axis = [(x, 0), (y, 1)].into_iter().find(|&(i, _)| i == Some(d));
            if let (Some((_, axis)), Some((a0, a))) = (axis, affine) {
                return layout.affine_placement(axis, &a0, &a);
            }
            if let Some(c) = &chained {
                return (c.factor[d], c.offset[d]);
            }
            (shape_ratio(base_var.shape[d], v.shape[d]), 0.0)
        })
        .unzip()
}

/// `base / len`, rounded to a whole factor when within one cell of it
/// (levels built with trimmed or kept partial windows).
fn shape_ratio(base: u64, len: u64) -> f64 {
    if len == 0 {
        return f64::NAN;
    }
    let ratio = base as f64 / len as f64;
    let whole = ratio.round().max(1.0);
    if (whole * len as f64 - base as f64).abs() <= whole {
        whole
    } else {
        ratio
    }
}
