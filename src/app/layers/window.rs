//! The window an overlay reads: the base layer's roles, ranges and indices,
//! given to the overlay's dimensions of the same name.

use super::VariableSelection;
use super::alignment::dim_named;
use crate::app::{AnimationRole, DimConfig};
use crate::plots::PlotType;

/// The selection of variable `var_idx` of `dataset` (a staged selection) that
/// reads the `base` layer's window: dimensions named like the base's take its
/// roles, ranges and indices (clamped to the variable), others one index.
pub fn overlay_selection(
    base: &VariableSelection,
    dataset: &VariableSelection,
    var_idx: usize,
) -> Option<VariableSelection> {
    let meta = dataset.metadata.as_ref()?;
    meta.variables.get(var_idx)?;
    base.variable_info()?;
    let mut selection = VariableSelection {
        store_kind: dataset.store_kind,
        store_target: dataset.store_target.clone(),
        metadata: Some(meta.clone()),
        metadata_generation: dataset.metadata_generation,
        variable_idx: var_idx,
        dim_config: Vec::new(),
        dim_indices: Vec::new(),
        dim_ranges: Vec::new(),
        spatial_dims: Vec::new(),
        animated_dim: None,
        plot_type: PlotType::Heatmap,
    };
    follow_base_window(base, &mut selection);
    Some(selection)
}

/// Gives `overlay`'s dimensions the `base` window in place: roles, ranges
/// and indices of the base's dimension of the same name (clamped to the
/// overlay's sizes), one index 0 for the others. Whether the window moved:
/// other roles, ranges or fixed indices (the animated index alone, a playback
/// step, does not count). Allocates only when the roles change.
pub fn follow_base_window(base: &VariableSelection, overlay: &mut VariableSelection) -> bool {
    let (Some(base_var), Some(var)) = (
        base.variable_info(),
        overlay
            .metadata
            .as_ref()
            .and_then(|m| m.variables.get(overlay.variable_idx)),
    ) else {
        return false;
    };
    let rank = var.shape.len();
    let mut changed = overlay.dim_config.len() != rank;
    overlay.dim_config.resize(rank, DimConfig::default());
    overlay.dim_indices.resize(rank, 0);
    overlay.dim_ranges.resize(rank, (0, 0));
    for (i, name) in var.dimension_names.iter().enumerate().take(rank) {
        let last = var
            .shape
            .get(i)
            .map_or(0, |&n| (n as usize).saturating_sub(1));
        let (config, index, range) = base_dim(base, base_var, name, last);
        let old = (
            overlay.dim_config[i],
            overlay.dim_indices[i],
            overlay.dim_ranges[i],
        );
        let roles = |c: DimConfig| (c.spatial, c.animation, c.active);
        let animated = config.animation == AnimationRole::Animated;
        changed |= roles(old.0) != roles(config) || old.2 != range || (!animated && old.1 != index);
        overlay.dim_config[i] = config;
        overlay.dim_indices[i] = index;
        overlay.dim_ranges[i] = range;
    }
    if changed {
        overlay.spatial_dims = DimConfig::spatial_dims(&overlay.dim_config);
        overlay.animated_dim = DimConfig::animated_dim(&overlay.dim_config);
    }
    changed
}

/// The role, index and range the overlay's dimension `name` (of last index
/// `last`) takes from the base's dimension of that name, clamped to it; one
/// index 0 when the base has none.
fn base_dim(
    base: &VariableSelection,
    base_var: &crate::data::VariableInfo,
    name: &str,
    last: usize,
) -> (DimConfig, usize, (usize, usize)) {
    let Some(b) = dim_named(base_var, name) else {
        return (DimConfig::default(), 0, (0, 0));
    };
    let clamp = |(s, e): (usize, usize)| (s.min(last), e.min(last));
    let config = base
        .dim_config
        .get(b)
        .map_or(DimConfig::default(), |c| DimConfig {
            range: clamp(c.range),
            index: c.index.min(last),
            ..*c
        });
    let index = base.dim_indices.get(b).map_or(0, |&n| n.min(last));
    let range = base.dim_ranges.get(b).map_or((0, 0), |&r| clamp(r));
    (config, index, range)
}
