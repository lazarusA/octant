//! Visualization plot type transitions and dimension layout adaptation.

use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::plots::PlotType;

impl OctantApp {
    /// Switches the active visualization plot type, adapting dimension layouts
    /// between 2D planar and 3D volumetric representations, invalidating
    /// mismatched cross-pipeline data, and loading the block for the new view.
    pub fn switch_plot_type(&mut self, new_plot_type: PlotType) {
        if self.active_plot_type == new_plot_type {
            return;
        }
        let prev_plot_type = self.active_plot_type;
        self.active_plot_type = new_plot_type;

        let was_3d = prev_plot_type == PlotType::Volume || prev_plot_type == PlotType::PointCloud;
        let is_3d = new_plot_type == PlotType::Volume || new_plot_type == PlotType::PointCloud;

        if was_3d && !is_3d {
            // Switching from 3D Volume to 2D / surface:
            // 1. Collapse spatial Z to a single slice index, unless it is the animated dimension:
            for (z_idx, c) in self.dim_config.iter_mut().enumerate() {
                if c.spatial == SpatialRole::Z {
                    if c.animation == AnimationRole::Animated {
                        c.spatial = SpatialRole::None;
                        c.active = true;
                    } else {
                        let current_z = self.selected_dim_indices.get(z_idx).copied().unwrap_or(0);
                        c.spatial = SpatialRole::None;
                        c.active = false;
                        c.range = (current_z, current_z);
                        if z_idx < self.selected_dim_ranges.len() {
                            self.selected_dim_ranges[z_idx] = (current_z, current_z);
                        }
                        if z_idx < self.plotted_selected_dim_ranges.len() {
                            self.plotted_selected_dim_ranges[z_idx] = (current_z, current_z);
                        }
                    }
                }
            }
            for (z_idx, c) in self.plotted_dim_config.iter_mut().enumerate() {
                if c.spatial == SpatialRole::Z {
                    if c.animation == AnimationRole::Animated {
                        c.spatial = SpatialRole::None;
                        c.active = true;
                    } else {
                        let current_z = self
                            .plotted_selected_dim_indices
                            .get(z_idx)
                            .copied()
                            .unwrap_or(0);
                        c.spatial = SpatialRole::None;
                        c.active = false;
                        c.range = (current_z, current_z);
                    }
                }
            }
            self.spatial_dims = crate::app::DimConfig::spatial_dims(&self.dim_config);
            self.plotted_spatial_dims =
                crate::app::DimConfig::spatial_dims(&self.plotted_dim_config);

            // 2. Clear stale 2D matrix/renderers if they don't match current variable:
            let cur_var = self
                .plotted_variable_info()
                .or_else(|| self.selected_variable_info());
            let cur_name = cur_var.map(|v| v.name.as_str());
            if self
                .matrix_data
                .as_ref()
                .is_some_and(|m| cur_name.is_none_or(|n| !m.dataset_name.contains(n)))
            {
                self.matrix_data = None;
                self.renderer = None;
                self.sphere_renderer = None;
                self.surface_renderer = None;
                self.line_renderer = None;
            }

            self.lock_color_bounds = false;
        } else if !was_3d && is_3d {
            // Switching from 2D / surface to 3D Volume:
            // 1. Expand spatial Z to full range:
            let fallback_anim = if self.dim_config.len() >= 3 {
                self.animated_dim
            } else {
                None
            };
            let z_idx_opt = self
                .dim_config
                .iter()
                .position(|c| c.spatial == SpatialRole::Z && c.animation != AnimationRole::Animated)
                .or_else(|| {
                    self.dim_config.iter().position(|c| {
                        c.spatial == SpatialRole::None && c.animation != AnimationRole::Animated
                    })
                })
                .or(fallback_anim);
            if let Some(z_idx) = z_idx_opt {
                self.dim_config[z_idx].spatial = SpatialRole::Z;
                self.dim_config[z_idx].active = true;
                if self.dim_config[z_idx].animation != AnimationRole::Animated {
                    let max_z = self
                        .plotted_variable_info()
                        .or_else(|| self.selected_variable_info())
                        .and_then(|v| v.shape.get(z_idx))
                        .map(|&s| (s as usize).saturating_sub(1))
                        .unwrap_or(0);
                    self.dim_config[z_idx].range = (0, max_z);
                    if z_idx < self.selected_dim_ranges.len() {
                        self.selected_dim_ranges[z_idx] = (0, max_z);
                    }
                    if z_idx < self.plotted_selected_dim_ranges.len() {
                        self.plotted_selected_dim_ranges[z_idx] = (0, max_z);
                    }
                }
            }
            self.spatial_dims = crate::app::DimConfig::spatial_dims(&self.dim_config);
            self.plotted_spatial_dims =
                crate::app::DimConfig::spatial_dims(&self.plotted_dim_config);

            // 2. Clear stale volume data if it doesn't match current variable:
            let cur_var = self
                .plotted_variable_info()
                .or_else(|| self.selected_variable_info());
            let cur_name = cur_var.map(|v| v.name.as_str());
            if self
                .volume_data
                .as_ref()
                .is_some_and(|v| cur_name.is_none_or(|n| !v.dataset_name.contains(n)))
            {
                self.volume_data = None;
                self.volume_renderer = None;
                self.point_cloud_renderer = None;
            }

            self.lock_color_bounds = false;
        }

        if !self.is_exploring_unplotted_variable() {
            self.load_selected_variable_block();
        }
    }
}
