//! Aspect ratio scaling, viewport sizing, and spatial dimension resolution.

use crate::app::OctantApp;

impl OctantApp {
    /// Computes the 3D bounding aspect ratio `(aspect_x, aspect_y, aspect_z)` for 3D plots.
    pub fn get_3d_aspect_ratio(&self) -> (f32, f32, f32) {
        let (mut scale_x, mut scale_y, mut scale_z) = (1.0f32, 1.0f32, 1.0f32);
        if let Some(var) = self
            .plotted_variable_info()
            .or_else(|| self.selected_variable_info())
        {
            let get_scale = |axis_name: &str| -> Option<f32> {
                for (k, v) in &var.attributes {
                    let is_match = k.eq_ignore_ascii_case(axis_name)
                        || (k.len() == 6 + axis_name.len()
                            && k[..6].eq_ignore_ascii_case("scale_")
                            && k[6..].eq_ignore_ascii_case(axis_name));
                    if is_match
                        && let Ok(val) = v.parse::<f32>()
                        && val > 0.0
                    {
                        return Some(val);
                    }
                }
                None
            };

            let x_name = self.get_spatial_dim_name(0);
            let y_name = self.get_spatial_dim_name(1);
            let z_name = self.get_spatial_dim_name(2);

            if let Some(sx) = x_name
                .as_deref()
                .and_then(get_scale)
                .or_else(|| get_scale("x"))
            {
                scale_x = sx;
            }
            if let Some(sy) = y_name
                .as_deref()
                .and_then(get_scale)
                .or_else(|| get_scale("y"))
            {
                scale_y = sy;
            }
            if let Some(sz) = z_name
                .as_deref()
                .and_then(get_scale)
                .or_else(|| get_scale("z"))
            {
                scale_z = sz;
            }
        }

        let z_mult = self.plot_configs.volume.z_scale.clamp(0.01, 50.0);

        if let Some(vdata) = &self.layers.base.data.volume {
            let w = vdata.width as f32 * scale_x;
            let h = vdata.height as f32 * scale_y;
            let d = vdata.depth as f32 * scale_z * z_mult;
            let max_dim = w.max(h).max(d).max(1.0);
            return (w / max_dim, h / max_dim, d / max_dim);
        }

        let sel_ranges = &self.selected.dim_ranges;

        let x_idx = self.get_spatial_dim_index(0);
        let y_idx = self.get_spatial_dim_index(1);
        let z_idx = self.get_spatial_dim_index(2);

        let get_extent = |dim: usize| -> usize {
            if let Some(&(start, end)) = sel_ranges.get(dim) {
                (end + 1).saturating_sub(start).max(1)
            } else if let Some(meta) = self.selected.metadata.as_ref()
                && let Some(var) = meta.variables.get(self.selected.variable_idx)
                && let Some(&s) = var.shape.get(dim)
            {
                (s as usize).max(1)
            } else {
                64
            }
        };

        let w = (get_extent(x_idx) as f32 * scale_x).max(1.0);
        let h = (get_extent(y_idx) as f32 * scale_y).max(1.0);
        let d = (get_extent(z_idx) as f32 * scale_z * z_mult).max(1.0);
        let max_dim = w.max(h).max(d).max(1.0);

        (w / max_dim, h / max_dim, d / max_dim)
    }

    /// Computes circular shift offsets (shift_x, shift_y, shift_z) along the animated spatial dimension.
    pub fn get_volume_shifts(&self) -> (u32, u32, u32) {
        let Some(anim_dim) = self.effective_animated_dim() else {
            return (0, 0, 0);
        };

        let dim_configs = self.effective_dim_config();
        let spatial_role = dim_configs
            .get(anim_dim)
            .map(|c| c.spatial)
            .unwrap_or(crate::app::SpatialRole::None);

        let (width, height, depth) = if let Some(vdata) = &self.layers.base.data.volume {
            (
                vdata.width.max(1) as u32,
                vdata.height.max(1) as u32,
                vdata.depth.max(1) as u32,
            )
        } else {
            (64, 64, 64)
        };

        let origin = self
            .effective_selected_dim_ranges()
            .get(anim_dim)
            .map(|r| r.0)
            .unwrap_or(0);

        let local_step = (self.playback.current_timestep.saturating_sub(origin)) as u32;

        match spatial_role {
            crate::app::SpatialRole::X | crate::app::SpatialRole::Grid => {
                (local_step % width, 0, 0)
            }
            crate::app::SpatialRole::Y => (0, local_step % height, 0),
            crate::app::SpatialRole::Z => (0, 0, local_step % depth),
            crate::app::SpatialRole::None => (0, 0, 0),
        }
    }

    /// Resolves the metadata dimension index for a given spatial axis (0 = X, 1 = Y, 2 = Z).
    pub fn get_spatial_dim_index(&self, axis: usize) -> usize {
        let meta = self.effective_dataset_metadata();
        let var_idx = if self.plotted().metadata.is_some() {
            self.plotted().variable_idx
        } else {
            self.selected.variable_idx
        };
        let configs = self.effective_dim_config();
        if let Some(meta) = meta
            && let Some(var) = meta.variables.get(var_idx)
        {
            let (x, y, z) = Self::resolve_spatial_axes(
                var.shape.len(),
                &var.dimension_names,
                &var.dimension_names,
                configs,
            );
            match axis {
                0 => x,
                1 => y,
                _ => z,
            }
        } else {
            axis
        }
    }

    /// Resolves the metadata dimension name for spatial axis (0 = X, 1 = Y, 2 = Z).
    pub fn get_spatial_dim_name(&self, axis: usize) -> Option<String> {
        let idx = self.get_spatial_dim_index(axis);
        let meta = self.effective_dataset_metadata();
        let var_idx = if self.plotted().metadata.is_some() {
            self.plotted().variable_idx
        } else {
            self.selected.variable_idx
        };
        meta.and_then(|m| m.variables.get(var_idx))
            .and_then(|v| v.dimension_names.get(idx).cloned())
    }

    /// Returns the effective original (width, height) of the active 2D data or pyramid.
    pub fn active_data_dimensions_2d(&self) -> (usize, usize) {
        self.layers.base.data.dimensions_2d()
    }

    /// Returns the aspect ratio (width / height) of the active 2D dataset.
    pub fn data_aspect_ratio_2d(&self) -> f32 {
        if let Some(m) = &self.layers.base.data.matrix
            && m.grid.is_healpix()
        {
            return 2.0;
        }
        let (w, h) = self.active_data_dimensions_2d();
        (w as f32 / (h as f32).max(1.0)).max(0.001)
    }

    /// Computes data aspect scaling factors [scale_x, scale_y] to preserve proportional aspect framing.
    pub fn compute_aspect_scale(&self, canvas_size: egui::Vec2) -> [f32; 2] {
        if self.layout.enforce_data_aspect_ratio && self.layers.base.data.matrix.is_some() {
            let data_aspect = self.data_aspect_ratio_2d();
            let canvas_aspect = canvas_size.x / canvas_size.y.max(1.0);
            if canvas_aspect > data_aspect {
                [data_aspect / canvas_aspect, 1.0]
            } else {
                [1.0, canvas_aspect / data_aspect]
            }
        } else {
            [1.0, 1.0]
        }
    }
}
