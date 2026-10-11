//! Visualization plot type transitions and dimension layout adaptation.

use crate::app::{AnimationRole, OctantApp, SpatialRole};
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
        let channel_dim = if self.layers.base.composite.enabled {
            self.channel_dim_index()
        } else {
            None
        };
        let max_ch = channel_dim
            .and_then(|ch| {
                self.plotted_variable_info()
                    .or_else(|| self.selected_variable_info())
                    .and_then(|v| v.shape.get(ch))
                    .map(|&s| (s as usize).saturating_sub(1))
            })
            .unwrap_or(0);

        adapt_sel_to_2d(&mut self.selected, channel_dim, max_ch);
        adapt_sel_to_2d(self.layers.base.selection_mut(), channel_dim, max_ch);
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
        let channel_dim = if self.layers.base.composite.enabled {
            self.channel_dim_index()
        } else {
            None
        };
        let fallback_anim = if self.selected.dim_config.len() >= 3 {
            self.selected
                .animated_dim
                .filter(|&d| Some(d) != channel_dim)
        } else {
            None
        };
        let z_idx_opt = self
            .selected
            .dim_config
            .iter()
            .enumerate()
            .position(|(i, c)| {
                c.spatial == SpatialRole::Z
                    && c.animation != AnimationRole::Animated
                    && Some(i) != channel_dim
            })
            .or_else(|| {
                self.selected
                    .dim_config
                    .iter()
                    .enumerate()
                    .position(|(i, c)| {
                        c.spatial == SpatialRole::None
                            && c.animation != AnimationRole::Animated
                            && Some(i) != channel_dim
                    })
            })
            .or(fallback_anim);
        if let Some(z_idx) = z_idx_opt {
            let max_z = if self.selected.dim_config[z_idx].animation != AnimationRole::Animated {
                let max = self
                    .plotted_variable_info()
                    .or_else(|| self.selected_variable_info())
                    .and_then(|v| v.shape.get(z_idx))
                    .map(|&s| (s as usize).saturating_sub(1))
                    .unwrap_or(0);
                Some(max)
            } else {
                None
            };
            adapt_sel_to_3d(&mut self.selected, z_idx, max_z);
            adapt_sel_to_3d(self.layers.base.selection_mut(), z_idx, max_z);
        }
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

fn adapt_sel_to_2d(
    sel: &mut crate::app::VariableSelection,
    channel_dim: Option<usize>,
    max_ch: usize,
) {
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
    if let Some(ch_idx) = channel_dim {
        if let Some(c) = sel.dim_config.get_mut(ch_idx) {
            c.range = (0, max_ch);
            c.active = true;
        }
        if let Some(r) = sel.dim_ranges.get_mut(ch_idx) {
            *r = (0, max_ch);
        }
    }
    sel.spatial_dims = crate::app::DimConfig::spatial_dims(&sel.dim_config);
}

fn adapt_sel_to_3d(sel: &mut crate::app::VariableSelection, z_idx: usize, max_z: Option<usize>) {
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
