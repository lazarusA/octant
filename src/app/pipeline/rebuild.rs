//! GPU render pipeline initialization, caching, and buffer allocation.

use crate::app::OctantApp;
use crate::data::matrix_data::MatrixData;
use crate::data::volume_data::VolumeData;
use crate::plots::{
    Coastline3DRenderer, CoastlineRenderer, LineRenderer, MatrixRenderer, PlotType,
    PointCloudRenderer, SphereRenderer, SurfaceRenderer, VolumeRenderer,
};
use std::sync::Arc;

impl OctantApp {
    /// Rebuilds or updates existing GPU buffers for 2D matrix data.
    pub fn rebuild_pipeline_with_matrix_data(&mut self, data: MatrixData) {
        let total_elements = data.width.saturating_mul(data.height);

        let var_key = format!(
            "{}:{}",
            self.plotted_store_target_input, self.plotted_variable_idx
        );
        let is_new_variable = self.current_plotted_var_key.as_ref() != Some(&var_key);

        if is_new_variable {
            self.current_plotted_var_key = Some(var_key);
            self.global_data_min = data.min_val;
            self.global_data_max = data.max_val;
            self.color_range_min = data.min_val;
            self.color_range_max = data.max_val;
            self.volume_cmin = data.min_val;
            self.volume_cmax = data.max_val;
            self.lock_color_bounds = false;
        } else {
            self.global_data_min = self.global_data_min.min(data.min_val);
            self.global_data_max = self.global_data_max.max(data.max_val);

            if !self.lock_color_bounds {
                self.color_range_min = data.min_val;
                self.color_range_max = data.max_val;
                self.volume_cmin = data.min_val;
                self.volume_cmax = data.max_val;
            }
        }

        let is_oversized = total_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
        if is_oversized {
            self.enable_pyramid_resampling = true;
        }

        if data.height == 1 {
            self.line_plot_all_series = false;
            self.line_profile_dim_idx = 0;
            self.line_profile_slice_idx = 0;
        }

        let effective_data = if data.height > 1 && (self.enable_pyramid_resampling || is_oversized)
        {
            let pyramid = Arc::new(crate::data::MatrixPyramid::new(
                &data.values,
                data.width,
                data.height,
                &data.dataset_name,
                self.pyramid_aggregation_op,
                512,
            ));
            self.resampler.set_pyramid(Some(pyramid.clone()));
            self.active_pyramid = Some(pyramid.clone());

            let aspect_scale = [1.0f32, 1.0f32];
            let ((u_min, u_max), (v_min, v_max)) =
                crate::data::ViewportResampler::compute_visible_data_bounds(
                    [self.heatmap_pan.x, self.heatmap_pan.y],
                    self.heatmap_zoom,
                    aspect_scale,
                );

            let (target_w, target_h) = crate::data::ViewportResampler::compute_target_resolution(
                data.width,
                data.height,
                2048,
            );
            pyramid.sample_viewport((u_min, u_max), (v_min, v_max), (target_w, target_h))
        } else {
            self.active_pyramid = None;
            self.resampler.set_pyramid(None);
            data.clone()
        };

        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let same_grid = self
                .matrix_data
                .as_ref()
                .is_some_and(|m| m.grid.same_geometry(&effective_data.grid));
            let same_dimensions = !is_new_variable
                && same_grid
                && self.matrix_data.as_ref().is_some_and(|m| {
                    m.width == effective_data.width && m.height == effective_data.height
                });
            let can_have_3d_surface =
                total_elements <= crate::plots::common::MAX_2D_SURFACE_ELEMENTS;
            let surface_renderers_ready = !can_have_3d_surface
                || (self.sphere_renderer.is_some() && self.surface_renderer.is_some());

            if same_dimensions
                && self.renderer.is_some()
                && self.line_renderer.is_some()
                && surface_renderers_ready
            {
                if let Some(renderer) = &self.renderer {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                if let Some(sphere_renderer) = &self.sphere_renderer {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        sphere_renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    sphere_renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                if let Some(surface_renderer) = &self.surface_renderer {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        surface_renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    surface_renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                if let Some(coastline_renderer) = &self.coastline_3d_renderer {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        coastline_renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    coastline_renderer
                        .update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                if let Some(line_renderer) = &self.line_renderer {
                    line_renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
            } else {
                let coord_x = effective_data.grid.coords_x();
                let coord_y = effective_data.grid.coords_y();
                let renderer = MatrixRenderer::new_with_coords(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &effective_data.values,
                    effective_data.width,
                    effective_data.height,
                    coord_x,
                    coord_y,
                );
                let line_renderer = LineRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &effective_data.values,
                    effective_data.width,
                    effective_data.height,
                );
                self.renderer = Some(Arc::new(renderer));
                self.line_renderer = Some(Arc::new(line_renderer));

                if total_elements <= crate::plots::common::MAX_2D_SURFACE_ELEMENTS {
                    let coord_x = effective_data.grid.coords_x();
                    let coord_y = effective_data.grid.coords_y();
                    let sphere_renderer = SphereRenderer::new_sphere_with_coords(
                        &wgpu_render_state.device,
                        wgpu_render_state.target_format,
                        &effective_data.values,
                        effective_data.width,
                        effective_data.height,
                        coord_x,
                        coord_y,
                    );
                    let surface_renderer = SurfaceRenderer::new_surface_with_coords(
                        &wgpu_render_state.device,
                        wgpu_render_state.target_format,
                        &effective_data.values,
                        effective_data.width,
                        effective_data.height,
                        coord_x,
                        coord_y,
                    );
                    let coastline_buffer = crate::plots::load_coastline(self.coastline_current_lod);
                    let coastline_3d_renderer = Coastline3DRenderer::new(
                        &wgpu_render_state.device,
                        wgpu_render_state.target_format,
                        coastline_buffer.as_slice(),
                        &effective_data.values,
                        effective_data.width,
                        effective_data.height,
                        coord_x,
                        coord_y,
                    );
                    self.sphere_renderer = Some(Arc::new(sphere_renderer));
                    self.surface_renderer = Some(Arc::new(surface_renderer));
                    self.coastline_3d_renderer = Some(Arc::new(coastline_3d_renderer));
                } else {
                    self.sphere_renderer = None;
                    self.surface_renderer = None;
                    self.coastline_3d_renderer = None;
                }

                if data.height == 1 {
                    self.active_plot_type = PlotType::Line;
                }
            }
        }

        // --- Coastline renderer: initialise once, upgrade LOD in background ---
        if self.coastline_renderer.is_none()
            && let Some(wgpu_render_state) = &self.wgpu_render_state
        {
            let buf = crate::plots::load_coastline(crate::plots::CoastlineLod::Lod110m);
            let r = CoastlineRenderer::new(
                &wgpu_render_state.device,
                wgpu_render_state.target_format,
                buf.as_slice(),
            );
            self.coastline_renderer = Some(Arc::new(r));
            self.coastline_current_lod = crate::plots::CoastlineLod::Lod110m;
        }

        self.matrix_data = Some(data);
    }

    /// Rebuilds or updates existing GPU buffers for 3D volume data.
    pub fn rebuild_pipeline_with_volume_data(&mut self, data: VolumeData) {
        if data.values.len() > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
            log::error!(
                "Volume data buffer size ({} elements / {} bytes) exceeds maximum GPU storage buffer limit ({})",
                data.values.len(),
                data.values.len() * 4,
                crate::plots::common::MAX_GPU_STORAGE_BUFFER_BYTES
            );
            return;
        }

        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let same_dimensions = self.volume_data.as_ref().is_some_and(|v| {
                v.width == data.width && v.height == data.height && v.depth == data.depth
            });

            if same_dimensions
                && self.volume_renderer.is_some()
                && self.point_cloud_renderer.is_some()
            {
                if let Some(volume_renderer) = &self.volume_renderer {
                    volume_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
                if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                    point_cloud_renderer.update_data(&wgpu_render_state.queue, &data.values);
                }
            } else {
                let volume_renderer = VolumeRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                let point_cloud_renderer = PointCloudRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                self.volume_renderer = Some(Arc::new(volume_renderer));
                self.point_cloud_renderer = Some(Arc::new(point_cloud_renderer));
            }
        }

        let var_key = format!(
            "{}:{}",
            self.plotted_store_target_input, self.plotted_variable_idx
        );
        let is_new_variable = self.current_plotted_var_key.as_ref() != Some(&var_key);

        if is_new_variable {
            self.current_plotted_var_key = Some(var_key);
            self.global_data_min = data.min_val;
            self.global_data_max = data.max_val;
            self.volume_cmin = data.min_val;
            self.volume_cmax = data.max_val;
            self.color_range_min = data.min_val;
            self.color_range_max = data.max_val;
            self.lock_color_bounds = false;
        } else {
            self.global_data_min = self.global_data_min.min(data.min_val);
            self.global_data_max = self.global_data_max.max(data.max_val);

            if !self.lock_color_bounds {
                self.volume_cmin = data.min_val;
                self.volume_cmax = data.max_val;
                self.color_range_min = data.min_val;
                self.color_range_max = data.max_val;
            }
        }

        self.volume_data = Some(data);
    }
}

// ---------------------------------------------------------------------------
// Coastline LOD reload (synchronous, user-initiated)
// ---------------------------------------------------------------------------

impl OctantApp {
    /// Loads the requested coastline LOD from disk (or the embedded 110m static)
    /// and immediately swaps it into the GPU vertex buffer.
    ///
    /// Call this when the user changes the LOD selector in Settings.
    /// The data is at most ~3 MB so the disk read is imperceptible.
    pub fn reload_coastline_lod(&mut self, lod: crate::plots::CoastlineLod) {
        if lod == self.coastline_current_lod {
            return;
        }
        let Some(renderer) = self.coastline_renderer.as_ref().map(Arc::clone) else {
            return;
        };
        let Some(wgpu_state) = &self.wgpu_render_state else {
            return;
        };

        let buf = crate::plots::load_coastline(lod);
        renderer.swap_vertices(&wgpu_state.device, &wgpu_state.queue, buf.as_slice());
        if let Some(renderer_3d) = self.coastline_3d_renderer.as_ref().map(Arc::clone) {
            renderer_3d.swap_vertices(&wgpu_state.device, &wgpu_state.queue, buf.as_slice());
        }
        self.coastline_current_lod = lod;
        log::info!(
            "Coastline LOD changed to {:?} ({} vertex pairs)",
            lod,
            buf.as_slice().len() / 2
        );
    }
}
