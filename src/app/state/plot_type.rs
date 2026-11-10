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
        // The slicer's axes mean other things on another plot type.
        self.plot_configs.slice.adapt_to(new_plot_type);

        let was_3d = prev_plot_type == PlotType::Volume || prev_plot_type == PlotType::PointCloud;
        let is_3d = new_plot_type == PlotType::Volume || new_plot_type == PlotType::PointCloud;

        if was_3d != is_3d {
            self.switch_dimensionality(is_3d);
        }
        // Those switches change the base layer's plot type in place: overlays
        // follow it now, not when its block arrives.
        self.refresh_alignments();

        if !self.is_exploring_unplotted_variable() {
            self.load_selected_variable_block();
        }
    }

    /// Moves the staged and plotted selections between 2D and 3D plots
    /// (`to_3d`), dropping the base layer's data of the other kind when it
    /// shows another variable.
    fn switch_dimensionality(&mut self, to_3d: bool) {
        // Each selection by its own variable: the staged one may be another.
        let composite = self.layers.base.composite.enabled;
        let adapt = if to_3d {
            adapt_sel_to_3d
        } else {
            adapt_sel_to_2d
        };
        adapt(&mut self.selected, composite);
        adapt(self.layers.base.selection_mut(), composite);
        self.layers.base.selection_mut().plot_type = self.selected.plot_type;

        let cur_var = self.effective_variable_info();
        let cur_name = cur_var.map(|v| v.name.as_str());
        let data = &self.layers.base.data;
        let shown = if to_3d {
            data.volume.as_ref().map(|v| v.dataset_name.as_str())
        } else {
            data.matrix.as_ref().map(|m| m.dataset_name.as_str())
        };
        let stale = shown.is_some_and(|d| cur_name.is_none_or(|n| !d.contains(n)));
        if !self.is_exploring_unplotted_variable() && stale {
            if to_3d {
                self.layers.base.clear_3d();
            } else {
                self.layers.base.clear_2d();
            }
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
    for z_idx in 0..sel.dim_config.len() {
        let c = &mut sel.dim_config[z_idx];
        if c.spatial != SpatialRole::Z {
            continue;
        }
        c.spatial = SpatialRole::None;
        c.active = c.animation == AnimationRole::Animated || Some(z_idx) == channel_dim;
        if !c.active {
            let current_z = sel.dim_config.get(z_idx).map(|c| c.index).unwrap_or(0);
            sel.set_index(z_idx, current_z);
        }
    }
    // A composite's channels keep their selected range.
    if let Some(c) = channel_dim.and_then(|ch| sel.dim_config.get_mut(ch)) {
        c.active = true;
    }
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
        .then_some(sel.animated_dim())
        .flatten()
        .filter(|&d| Some(d) != channel_dim);
    free(SpatialRole::Z)
        .or_else(|| free(SpatialRole::None))
        .or(fallback_anim)
}

/// Gives `sel`'s stacking dimension (`z_dim_of`) the Z role, keeping its
/// selected range: one plane draws a one-slice volume until it is widened.
fn adapt_sel_to_3d(sel: &mut VariableSelection, composite: bool) {
    let channel_dim = channel_of(sel, composite);
    if let Some(c) = z_dim_of(sel, channel_dim).and_then(|z| sel.dim_config.get_mut(z)) {
        c.spatial = SpatialRole::Z;
        c.active = true;
    }
}
