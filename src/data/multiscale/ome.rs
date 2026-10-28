//! OME-Zarr images as multiscale stacks.
//!
//! OME-Zarr gives each level an absolute scale and translation per axis:
//! the physical position of index `i` (a pixel center) is
//! `origin + i * scale` ([`OmeLevel::origin`]: the translation, or half a
//! block in for coarser levels written with level 0's). A level's factor is
//! its scale over level 0's, and its offset puts its first cell's lower edge
//! on level 0's index grid.

use crate::data::metadata::VariableInfo;
use crate::data::ome::model::{OmeImage, OmeLevel};

use super::types::ScaleLevel;

/// Links the level variables of `image` (as built by OME discovery, each
/// with its `OmeRef`) as one stack. Levels that fell back to plain Zarr are
/// left out; nothing is linked when fewer than two remain or the metadata
/// gives no usable scale.
pub fn link_image(image: &OmeImage, variables: &mut [VariableInfo]) {
    let mut found: Vec<(usize, usize)> = variables
        .iter()
        .enumerate()
        .filter_map(|(i, v)| Some((v.ome.as_ref()?.level, i)))
        .collect();
    found.sort_unstable();
    let Some(&(base_level, base_var)) = found.first() else {
        return;
    };
    let Some(base) = image.levels.get(base_level) else {
        return;
    };
    let rank = variables[base_var].shape.len();
    let mut levels = Vec::with_capacity(found.len());
    for &(level, var) in &found {
        let Some(meta) = image.levels.get(level) else {
            continue;
        };
        let Some((factor, offset)) = placement(base, meta, rank) else {
            log::warn!(
                "OME-Zarr image '{}': level '{}' has no usable scale; its levels stay separate",
                image.group,
                meta.path
            );
            return;
        };
        let v = &variables[var];
        levels.push(ScaleLevel {
            path: v.name.clone(),
            shape: v.shape.clone(),
            factor,
            offset,
        });
    }
    let dims = variables[base_var].dimension_names.clone();
    super::try_link(stack_name(image), dims, levels, variables);
}

/// The image's name, else its group's last segment.
fn stack_name(image: &OmeImage) -> String {
    if let Some(name) = image.name.as_deref().filter(|n| !n.trim().is_empty()) {
        return name.to_string();
    }
    let group = image.group.trim_matches('/');
    group.rsplit('/').next().unwrap_or(group).to_string()
}

/// Factor and offset per axis of `level` relative to `base`, from each one's
/// [`OmeLevel::origin`]; a missing, zero or non-finite scale gives `None`.
fn placement(base: &OmeLevel, level: &OmeLevel, rank: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    let mut factor = Vec::with_capacity(rank);
    let mut offset = Vec::with_capacity(rank);
    for d in 0..rank {
        let (s0, s) = (*base.scale.get(d)?, *level.scale.get(d)?);
        if !s0.is_finite() || !s.is_finite() || s0 == 0.0 || s == 0.0 {
            return None;
        }
        let (t0, t) = (base.origin(base, d), level.origin(base, d));
        let f = (s / s0).abs();
        factor.push(f);
        // Lower edge of the level's cell 0, in level-0 cells from level 0's
        // lower edge (pixel centers sit half a cell above edges).
        let edge = (t - 0.5 * s) - (t0 - 0.5 * s0);
        offset.push(edge / s0);
    }
    Some((factor, offset))
}
