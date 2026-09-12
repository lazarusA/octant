//! Aspect ratio scaling, viewport sizing, and spatial dimension resolution.

use crate::app::OctantApp;

impl OctantApp {
    /// Computes the 3D bounding aspect ratio `(aspect_x, aspect_y, aspect_z)` for 3D plots.
    pub fn get_3d_aspect_ratio(&self) -> (f32, f32, f32) {
        if let Some(vdata) = &self.volume_data {
            let w = vdata.width as f32;
            let h = vdata.height as f32;
            let d = vdata.depth as f32;
            let max_dim = w.max(h).max(d).max(1.0);
            return (w / max_dim, h / max_dim, d / max_dim);
        }

        let (w, h, max_t) = self.matrix_data.as_ref().map_or((64, 64, 64), |m| {
            (m.width as u32, m.height as u32, m.max_timesteps as u32)
        });

        let (shape_d, shape_h, shape_w) = if let Some(meta) = &self.active_dataset_metadata {
            if let Some(v) = meta.variables.get(self.selected_variable_idx) {
                if v.shape.len() >= 3 {
                    (v.shape[0] as u32, v.shape[1] as u32, v.shape[2] as u32)
                } else {
                    (max_t, h, w)
                }
            } else {
                (max_t, h, w)
            }
        } else {
            (max_t, h, w)
        };

        let width = shape_w.max(w);
        let height = shape_h.max(h);
        let depth = shape_d.max(max_t);

        let max_spatial = (width.max(height)) as f32;
        let aspect_x = width as f32 / max_spatial;
        let aspect_y = height as f32 / max_spatial;
        let aspect_z = ((depth as f32 / max_spatial) * 0.12).clamp(0.4, 1.0);

        (aspect_x, aspect_y, aspect_z)
    }

    /// Computes circular shift offsets (shift_x, shift_y, shift_z) along the animated spatial dimension.
    pub fn get_volume_shifts(&self) -> (u32, u32, u32) {
        let Some(anim_dim) = self.plotted_animated_dim.or(self.animated_dim) else {
            return (0, 0, 0);
        };

        let dim_configs = if !self.plotted_dim_config.is_empty() {
            &self.plotted_dim_config
        } else {
            &self.dim_config
        };

        let spatial_role = dim_configs
            .get(anim_dim)
            .map(|c| c.spatial)
            .unwrap_or(crate::app::SpatialRole::None);

        let (width, height, depth) = if let Some(vdata) = &self.volume_data {
            (
                vdata.width.max(1) as u32,
                vdata.height.max(1) as u32,
                vdata.depth.max(1) as u32,
            )
        } else {
            (64, 64, 64)
        };

        let origin = self
            .active_slice_request
            .as_ref()
            .and_then(|req| req.selections.get(anim_dim))
            .map(|sel| match sel {
                crate::data::DimensionSelection::Range { start, .. } => *start,
                crate::data::DimensionSelection::Index(idx) => *idx,
            })
            .unwrap_or(0);

        let local_step = (self.current_timestep.saturating_sub(origin)) as u32;

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
        if let Some(meta) = &self.plotted_dataset_metadata
            && let Some(var) = meta.variables.get(self.plotted_variable_idx)
        {
            let (x, y, z) = Self::resolve_spatial_axes(
                var.shape.len(),
                &var.dimension_names,
                &var.dimension_names,
                &self.plotted_dim_config,
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
        self.plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.plotted_variable_idx))
            .and_then(|v| v.dimension_names.get(idx).cloned())
    }

    /// Returns a human-friendly label for the spatial axis (e.g., "Along X (lon)", "Along Z (depth)").
    pub fn get_spatial_dim_label(&self, axis: usize) -> String {
        let (name_opt, fallback) = match axis {
            0 => (self.get_spatial_dim_name(0), "X / Longitude"),
            1 => (self.get_spatial_dim_name(1), "Y / Latitude"),
            _ => (self.get_spatial_dim_name(2), "Z / Depth"),
        };

        if let Some(name) = name_opt {
            let axis_letter = match axis {
                0 => "X",
                1 => "Y",
                _ => "Z",
            };
            format!("Along {} ({})", axis_letter, name)
        } else {
            format!("Along {}", fallback)
        }
    }

    /// Returns the effective original (width, height) of the active 2D data or pyramid.
    pub fn active_data_dimensions_2d(&self) -> (usize, usize) {
        if let Some(pyr) = &self.active_pyramid {
            (pyr.original_width, pyr.original_height)
        } else if let Some(m) = &self.matrix_data {
            (m.width, m.height)
        } else {
            (1024, 1024)
        }
    }

    /// Returns the aspect ratio (width / height) of the active 2D dataset.
    pub fn data_aspect_ratio_2d(&self) -> f32 {
        if let Some(m) = &self.matrix_data
            && m.grid.is_healpix()
        {
            return 2.0;
        }
        let (w, h) = self.active_data_dimensions_2d();
        (w as f32 / (h as f32).max(1.0)).max(0.001)
    }

    /// Computes data aspect scaling factors [scale_x, scale_y] to preserve proportional aspect framing.
    pub fn compute_aspect_scale(&self, canvas_size: egui::Vec2) -> [f32; 2] {
        if self.enforce_data_aspect_ratio && self.matrix_data.is_some() {
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

    /// Resolves coordinate bounds and formatted title for a given dimension index.
    pub fn resolve_axis_bounds_and_title(
        &self,
        dim_idx: usize,
        fallback_name: &str,
        fallback_len: usize,
    ) -> ((f64, f64), String) {
        let mut bounds = (0.0, fallback_len.saturating_sub(1).max(1) as f64);
        let mut name = fallback_name.to_string();

        if let Some(meta) = &self.plotted_dataset_metadata
            && let Some(var) = meta.variables.get(self.plotted_variable_idx)
            && dim_idx < var.shape.len()
        {
            let dim_size = var
                .shape
                .get(dim_idx)
                .copied()
                .unwrap_or(fallback_len as u64) as usize;
            let (start_p, end_p) = self
                .plotted_selected_dim_ranges
                .get(dim_idx)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
            bounds = (start_p as f64, end_p as f64);

            if let Some(dim_n) = var.dimension_names.get(dim_idx) {
                name = dim_n.clone();
                if let Some(coord_bounds) = meta.get_coord_bounds_for_var_range(
                    Some(&var.name),
                    dim_n,
                    dim_size,
                    (start_p, end_p),
                ) {
                    bounds = coord_bounds;
                }
            }
        }

        let title = crate::utils::coordinates::format_dimension_axis_title(&name);
        (bounds, title)
    }
}
