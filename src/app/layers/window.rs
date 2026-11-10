//! The window an overlay reads: the base layer's roles, ranges and indices,
//! given to the overlay's dimensions of the same name.

use super::VariableSelection;
use super::alignment::dim_named;
use crate::app::{AnimationRole, DimConfig};
use crate::plots::PlotType;

/// The selection of variable `var_idx` of `dataset` (a staged selection) that
/// reads the `base` layer's window: dimensions named like the base's take its
/// roles, ranges and indices (clamped to the variable), others index 0.
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
        plot_type: PlotType::Heatmap,
        level: Default::default(),
        level0: None,
    };
    follow_base_window(base, &mut selection);
    Some(selection)
}

/// Gives `overlay`'s dimensions the `base` window in place: roles, ranges
/// and indices of the base's dimension of the same name (clamped to the
/// overlay's sizes); the others keep their index and range (0 when new; a
/// channel range follows the composite, `fit_overlay_channels`), with no role.
/// Whether the window moved:
/// other roles, ranges or fixed indices (a playback step, the animated index
/// or a collapsed animated range, does not count). Allocates only when the roles change.
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
    for (i, name) in var.dimension_names.iter().enumerate().take(rank) {
        let last = var
            .shape
            .get(i)
            .map_or(0, |&n| (n as usize).saturating_sub(1));
        let old = overlay.dim_config[i];
        // Dimensions the base lacks keep their own index and range, no role.
        let config = base_dim(base, base_var, name, last).unwrap_or(DimConfig {
            index: old.index.min(last),
            range: (old.range.0.min(last), old.range.1.min(last)),
            ..DimConfig::default()
        });
        let roles = |c: DimConfig| (c.spatial, c.animation, c.active);
        let animated = config.animation == AnimationRole::Animated;
        // A collapsed animated range is the playback step, like its index.
        let step_only = animated && !config.active;
        changed |= roles(old) != roles(config)
            || (!step_only && old.range != config.range)
            || (!animated && old.index != config.index);
        overlay.dim_config[i] = config;
    }
    changed
}

/// The role, index and range the overlay's dimension `name` (of last index
/// `last`) takes from the base's dimension of that name, clamped to it;
/// `None` when the base has none.
fn base_dim(
    base: &VariableSelection,
    base_var: &crate::data::VariableInfo,
    name: &str,
    last: usize,
) -> Option<DimConfig> {
    let b = dim_named(base_var, name)?;
    let config = base
        .dim_config
        .get(b)
        .map_or(DimConfig::default(), |c| DimConfig {
            range: (c.range.0.min(last), c.range.1.min(last)),
            index: c.index.min(last),
            ..*c
        });
    Some(config)
}
