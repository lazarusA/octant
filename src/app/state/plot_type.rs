//! Visualization plot type transitions and dimension layout adaptation.

use crate::app::state::dataset_activation::channel_dim;
use crate::app::{AnimationRole, OctantApp, SpatialRole, VariableSelection};
use crate::plots::PlotType;

impl OctantApp {
    /// Switches the active visualization plot type, adapting dimension layouts
    /// between 2D planar and 3D volumetric representations, invalidating
    /// mismatched cross-pipeline data, and loading the block for the new view.
    pub fn switch_plot_type(&mut self, new_plot_type: PlotType) {
        if self.selected.plot_type == new_plot_type {
            return;
        }
        let prev_plot_type = self.selected.plot_type;
        self.selected.plot_type = new_plot_type;

        let was_3d = prev_plot_type == PlotType::Volume || prev_plot_type == PlotType::PointCloud;
        let is_3d = new_plot_type == PlotType::Volume || new_plot_type == PlotType::PointCloud;

        if was_3d && !is_3d {
            self.switch_from_3d_to_2d();
        } else if !was_3d && is_3d {
            self.switch_from_2d_to_3d();
        }
        // Those switches change the base layer's plot type in place: overlays
        // follow it now, not when its block arrives.
        self.refresh_alignments();

        if !self.is_exploring_unplotted_variable() {
            self.load_selected_variable_block();
        }
    }

    fn switch_from_3d_to_2d(&mut self) {
        // Each selection by its own variable: the staged one may be another.
        let composite = self.layers.base.composite.enabled;
        adapt_sel_to_2d(&mut self.selected, composite);
        adapt_sel_to_2d(self.layers.base.selection_mut(), composite);
        self.layers.base.selection_mut().plot_type = self.selected.plot_type;

        let cur_var = self.plotted_variable_info();
        let cur_name = cur_var.map(|v| v.name.as_str());
        if !self.is_exploring_unplotted_variable()
            && self
                .layers
                .base
                .data
                .matrix
                .as_ref()
                .is_some_and(|m| cur_name.is_none_or(|n| !m.dataset_name.contains(n)))
        {
            self.layers.base.clear_2d();
        }

        self.layers.base.color.lock_bounds = false;
    }

    fn switch_from_2d_to_3d(&mut self) {
        // Each selection by its own variable: the staged one may be another.
        let composite = self.layers.base.composite.enabled;
        adapt_sel_to_3d(&mut self.selected, composite);
        adapt_sel_to_3d(self.layers.base.selection_mut(), composite);
        self.layers.base.selection_mut().plot_type = self.selected.plot_type;

        let cur_var = self.plotted_variable_info();
        let cur_name = cur_var.map(|v| v.name.as_str());
        if !self.is_exploring_unplotted_variable()
            && self
                .layers
                .base
                .data
                .volume
                .as_ref()
                .is_some_and(|v| cur_name.is_none_or(|n| !v.dataset_name.contains(n)))
        {
            self.layers.base.clear_3d();
        }

        self.layers.base.color.lock_bounds = false;
    }
}

/// The channel dimension of `sel`'s variable while composites are on.
fn channel_of(sel: &VariableSelection, composite: bool) -> Option<usize> {
    channel_dim(sel.variable_info().filter(|_| composite)?)
}

fn adapt_sel_to_2d(sel: &mut VariableSelection, composite: bool) {
    let channel_dim = channel_of(sel, composite);
    for (z_idx, c) in sel.dim_config.iter_mut().enumerate() {
        if c.spatial == SpatialRole::Z {
            c.spatial = SpatialRole::None;
            if c.animation == AnimationRole::Animated || Some(z_idx) == channel_dim {
                c.active = true;
            } else {
                let current_z = sel.dim_indices.get(z_idx).copied().unwrap_or(0);
                c.active = false;
                c.range = (current_z, current_z);
                if let Some(r) = sel.dim_ranges.get_mut(z_idx) {
                    *r = (current_z, current_z);
                }
            }
        }
    }
    // A composite's channels keep their selected range.
    if let Some(c) = channel_dim.and_then(|ch| sel.dim_config.get_mut(ch)) {
        c.active = true;
    }
    sel.spatial_dims = crate::app::DimConfig::spatial_dims(&sel.dim_config);
}

/// The dimension `sel` stacks along Z in 3D: an explicit Z, else the first
/// free one, else (3D and up) its animated one; never the channel.
fn z_dim_of(sel: &VariableSelection, channel_dim: Option<usize>) -> Option<usize> {
    let free = |role: SpatialRole| {
        (sel.dim_config.iter().enumerate()).position(|(i, c)| {
            c.spatial == role && c.animation != AnimationRole::Animated && Some(i) != channel_dim
        })
    };
    let fallback_anim = (sel.dim_config.len() >= 3)
        .then_some(sel.animated_dim)
        .flatten()
        .filter(|&d| Some(d) != channel_dim);
    free(SpatialRole::Z)
        .or_else(|| free(SpatialRole::None))
        .or(fallback_anim)
}

fn adapt_sel_to_3d(sel: &mut VariableSelection, composite: bool) {
    let channel_dim = channel_of(sel, composite);
    let Some(z_idx) = z_dim_of(sel, channel_dim) else {
        sel.spatial_dims = crate::app::DimConfig::spatial_dims(&sel.dim_config);
        return;
    };
    let animated = sel
        .dim_config
        .get(z_idx)
        .is_some_and(|c| c.animation == AnimationRole::Animated);
    let max_z = (!animated).then(|| {
        sel.variable_info()
            .and_then(|v| v.shape.get(z_idx))
            .map_or(0, |&s| (s as usize).saturating_sub(1))
    });
    if let Some(c) = sel.dim_config.get_mut(z_idx) {
        c.spatial = SpatialRole::Z;
        c.active = true;
    }
    if let Some(max) = max_z {
        if let Some(c) = sel.dim_config.get_mut(z_idx) {
            c.range = (0, max);
        }
        if let Some(r) = sel.dim_ranges.get_mut(z_idx) {
            *r = (0, max);
        }
    }
    sel.spatial_dims = crate::app::DimConfig::spatial_dims(&sel.dim_config);
}
