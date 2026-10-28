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
use super::types::{ScaleLevel, volume_of};

/// Links the levels of every GeoZarr pyramid among `variables`. Groups are
/// looked for at each variable's group and that group's parent (a level is
/// a child group or an array of the pyramid's group); attributes the reader
/// cannot answer are skipped.
pub fn link_pyramids(reader: &MetaReader<'_>, variables: &mut [VariableInfo]) {
    for group in pyramid_groups(variables) {
        let Ok(Some(attrs)) = reader.group_attributes(&group) else {
            continue;
        };
        if let Some(layout) = Layout::parse(&attrs) {
            link_layout(&group, &layout, variables);
        }
    }
    // Its attributes hold its groups' too: a zone group's affine (geoemb `utm_zones`).
    for v in variables.iter_mut().filter(|v| v.affine.is_none()) {
        v.affine = super::geozarr_layout::array_affine(&v.attributes, &v.dimension_names, &v.shape);
    }
}

/// Gives `variables` their ancestor groups' `spatial:` keys, nearest first,
/// for stores whose variables come from their arrays alone (web Icechunk);
/// run before [`link_pyramids`], whose affines read them.
pub fn inherit_group_spatial(reader: &MetaReader<'_>, variables: &mut [VariableInfo]) {
    use crate::utils::metadata::cf::{attr_string, is_spatial_key, merge_parent_attributes};
    for var in variables {
        for group in crate::utils::path::ancestor_paths(&var.name) {
            let Ok(Some(attrs)) = reader.group_attributes(group) else {
                continue;
            };
            let spatial = (attrs.iter().filter(|(k, _)| is_spatial_key(k)))
                .map(|(k, v)| (k.clone(), attr_string(v)))
                .collect();
            merge_parent_attributes(&mut var.attributes, &spatial);
        }
    }
}

/// [`link_pyramids`] over `storage`, the groups' attributes read
/// concurrently on the desktop (a `zarr.json`, then a `.zattrs`, per group).
pub fn link_pyramids_in(
    storage: std::sync::Arc<dyn zarrs::storage::ReadableStorageTraits>,
    variables: &mut [VariableInfo],
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let groups = pyramid_groups(variables);
        crate::data::ome::direct::walk_concurrent(storage, 3, |reader| {
            // Every group asked, so one walk names all the keys it needs.
            let pending = groups
                .iter()
                .filter(|g| reader.group_attributes(g).is_err())
                .count();
            link_pyramids(reader, variables);
            (pending == 0).then_some(())
        });
    }
    #[cfg(target_arch = "wasm32")]
    link_pyramids(&MetaReader::direct(storage), variables);
}

/// The groups a pyramid of `variables` may be described at: each
/// variable's group and that group's parent.
fn pyramid_groups(variables: &[VariableInfo]) -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    for v in variables {
        let group = v.group_path().unwrap_or("");
        let parent = group.rsplit_once('/').map_or("", |(p, _)| p);
        for g in [group, parent] {
            if !groups.iter().any(|known| known == g) {
                groups.push(g.to_string());
            }
        }
    }
    groups
}

/// Links one stack per array name in `layout`'s levels, each from the
/// finest level holding it (bands may start at a coarser level).
pub fn link_layout(group: &str, layout: &Layout, variables: &mut [VariableInfo]) {
    let mut names: Vec<String> = Vec::new();
    for entry in &layout.entries {
        let path = join(group, &entry.asset);
        for v in variables.iter_mut() {
            let Some(name) = relative_name(&v.name, &path) else {
                continue;
            };
            if !names.iter().any(|n| n == name) {
                names.push(name.to_string());
            }
            if v.shape.len() >= 2 {
                v.affine = layout.grid_affine(entry, &v.dimension_names).or(v.affine);
            }
        }
    }
    for name in names {
        let Some((base, levels)) = stack_levels(group, layout, &name, variables) else {
            continue;
        };
        let Some(v) = levels
            .first()
            .and_then(|first| variables.iter().find(|v| v.name == first.path))
        else {
            continue;
        };
        let dims = v.dimension_names.clone();
        // An array asset takes the pyramid group's name (else the asset's).
        let label = match (name.is_empty(), group.is_empty()) {
            (false, _) => name.as_str(),
            (true, false) => leaf(group),
            (true, true) => base.asset.as_str(),
        };
        super::try_link(label.to_string(), dims, levels, variables);
    }
}

/// `name` within level `asset_path`: `""` when the asset is the array
/// itself, the array's name when it is a direct child of the asset group.
pub(super) fn relative_name<'a>(name: &'a str, asset_path: &str) -> Option<&'a str> {
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

/// The variable named `name` at every level holding it with the rank of
/// the finest one (the stack's base, returned with them), finest first.
/// Scalars form no stack.
fn stack_levels<'l>(
    group: &str,
    layout: &'l Layout,
    name: &str,
    variables: &[VariableInfo],
) -> Option<(&'l Entry, Vec<ScaleLevel>)> {
    let find = |entry: &Entry| {
        let path = join(&join(group, &entry.asset), name);
        variables
            .iter()
            .find(|v| v.name.trim_matches('/') == path && !v.shape.is_empty())
    };
    let mut held: Vec<(&Entry, &VariableInfo)> = layout
        .entries
        .iter()
        .filter_map(|entry| Some((entry, find(entry)?)))
        .collect();
    held.sort_by_key(|(_, v)| std::cmp::Reverse(volume_of(&v.shape)));
    let &(base, base_var) = held.first()?;
    let mut levels: Vec<ScaleLevel> = held
        .iter()
        .filter(|(_, v)| v.shape.len() == base_var.shape.len())
        .map(|&(entry, v)| {
            let (factor, offset) = placement(layout, base, base_var, entry, v);
            ScaleLevel {
                path: v.name.clone(),
                shape: v.shape.clone(),
                factor,
                offset,
            }
        })
        .collect();
    levels.sort_by(|a, b| volume(&a.factor).total_cmp(&volume(&b.factor)));
    Some((base, levels))
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
    // Chains run from the layout's root: taken relative to `base`.
    let chained = layout
        .chain(entry, rank)
        .zip(layout.chain(base, rank))
        .map(|(c, b)| {
            let factor: Vec<f64> = c.factor.iter().zip(&b.factor).map(|(c, b)| c / b).collect();
            let offset: Vec<f64> = (0..rank)
                .map(|d| (c.offset[d] - b.offset[d]) / b.factor[d])
                .collect();
            (factor, offset)
        });
    let (x, y) = layout.spatial_dims(entry, &v.dimension_names);
    let affine = base.affine.zip(entry.affine);
    (0..rank)
        .map(|d| {
            let axis = [(x, 0), (y, 1)].into_iter().find(|&(i, _)| i == Some(d));
            if let (Some((_, axis)), Some((a0, a))) = (axis, affine) {
                return layout.affine_placement(axis, &a0, &a);
            }
            if let Some((factor, offset)) = &chained {
                return (factor[d], offset[d]);
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
