//! GPU render pipeline initialization, caching, and buffer allocation.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::matrix_data::MatrixData;
use crate::data::volume_data::VolumeData;
use crate::plots::{
    Coastline3DRenderer, CoastlineRenderer, LineRenderer, MatrixRenderer, PlotType,
    PointCloudRenderer, SphereRenderer, SurfaceRenderer, VolumeRenderer,
};
use std::sync::Arc;

impl OctantApp {
    /// Rebuilds or updates layer `id`'s GPU buffers for 2D matrix data.
    pub fn rebuild_pipeline_with_matrix_data(&mut self, id: LayerId, data: MatrixData) {
        let is_base = id == LayerId::BASE;
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let total_elements = data.width.saturating_mul(data.height);

        let var_key = format!(
            "{}:{}:2d",
            layer.selection().store_target,
            layer.selection().variable_idx
        );
        let is_new_variable = layer.data.var_key.as_ref() != Some(&var_key);

        if is_new_variable {
            layer.clear_3d();
            layer.data.var_key = Some(var_key);
            layer.color.global_min = data.min_val;
            layer.color.global_max = data.max_val;
            layer.color.range_min = data.min_val;
            layer.color.range_max = data.max_val;
            layer.color.lock_bounds = false;
        } else {
            layer.color.global_min = layer.color.global_min.min(data.min_val);
            layer.color.global_max = layer.color.global_max.max(data.max_val);

            if !layer.color.lock_bounds {
                layer.color.range_min = data.min_val;
                layer.color.range_max = data.max_val;
            }
        }

        let is_oversized = total_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
        if is_oversized {
            self.enable_pyramid_resampling = true;
        }

        if is_base && data.height == 1 {
            self.line_plot_all_series = false;
            self.line_profile_dim_idx = 0;
            self.line_profile_slice_idx = 0;
        }

        let is_rgb_composite = layer.composite.enabled
            || data.dataset_name.contains("RGB Composite")
            || data.dataset_name.contains("CMYK Composite");

        let effective_data = if !is_rgb_composite
            && data.height > 1
            && (self.enable_pyramid_resampling || is_oversized)
        {
            let pyramid = Arc::new(crate::data::MatrixPyramid::new(
                &data.values,
                data.width,
                data.height,
                &data.dataset_name,
                self.pyramid_aggregation_op,
                512,
            ));
            layer.data.resampler.set_pyramid(Some(pyramid.clone()));
            layer.data.pyramid = Some(pyramid.clone());

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
            layer.data.pyramid = None;
            layer.data.resampler.set_pyramid(None);
            data.clone()
        };

        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let same_grid = layer
                .data
                .matrix
                .as_ref()
                .is_some_and(|m| m.grid.same_geometry(&effective_data.grid));
            let same_dimensions = !is_new_variable
                && same_grid
                && layer.data.matrix.as_ref().is_some_and(|m| {
                    m.width == effective_data.width && m.height == effective_data.height
                });
            let can_have_3d_surface =
                total_elements <= crate::plots::common::MAX_2D_SURFACE_ELEMENTS;
            let renderers = &mut layer.renderers;
            let surface_renderers_ready =
                !can_have_3d_surface || (renderers.sphere.is_some() && renderers.surface.is_some());

            if same_dimensions
                && renderers.heatmap.is_some()
                && renderers.line.is_some()
                && surface_renderers_ready
            {
                if let Some(renderer) = &renderers.heatmap {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                if let Some(sphere_renderer) = &renderers.sphere {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        sphere_renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    sphere_renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                if let Some(surface_renderer) = &renderers.surface {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        surface_renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    surface_renderer.update_data(&wgpu_render_state.queue, &effective_data.values);
                }
                // Coastlines follow the base layer's grid.
                if is_base && let Some(coastline_renderer) = &self.coastline_3d_renderer {
                    if let (Some(cx), Some(cy)) = (
                        effective_data.grid.coords_x(),
                        effective_data.grid.coords_y(),
                    ) {
                        coastline_renderer.update_coords(&wgpu_render_state.queue, cx, cy);
                    }
                    coastline_renderer
                        .update_data(&wgpu_render_state.queue, &effective_data.values);
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
                renderers.heatmap = Some(Arc::new(renderer));
                renderers.line = Some(Arc::new(line_renderer));

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
                    renderers.sphere = Some(Arc::new(sphere_renderer));
                    renderers.surface = Some(Arc::new(surface_renderer));
                    if is_base {
                        let coastline_buffer =
                            crate::plots::load_coastline_sync(self.coastline_current_lod);
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
                        self.coastline_3d_renderer = Some(Arc::new(coastline_3d_renderer));
                    }
                } else {
                    renderers.sphere = None;
                    renderers.surface = None;
                    if is_base {
                        self.coastline_3d_renderer = None;
                    }
                }

                if data.height == 1 {
                    // The base layer's plot type is staged in `selected`.
                    let plot_type = if is_base {
                        &mut self.selected.plot_type
                    } else {
                        &mut layer.selection_mut().plot_type
                    };
                    if data.grid.is_healpix() {
                        if *plot_type == PlotType::Line {
                            *plot_type = PlotType::Heatmap;
                        }
                    } else {
                        *plot_type = PlotType::Line;
                    }
                }
            }
        }

        // --- Coastline renderer: initialise once, upgrade LOD in background ---
        if self.coastline_renderer.is_none()
            && let Some(wgpu_render_state) = &self.wgpu_render_state
        {
            let buf = crate::plots::load_coastline_sync(crate::plots::CoastlineLod::Lod110m);
            let r = CoastlineRenderer::new(
                &wgpu_render_state.device,
                wgpu_render_state.target_format,
                buf.as_slice(),
            );
            self.coastline_renderer = Some(Arc::new(r));
            self.coastline_current_lod = crate::plots::CoastlineLod::Lod110m;
        }

        layer.data.matrix = Some(data);
        layer.data.touch();
    }

    /// Rebuilds or updates layer `id`'s GPU buffers for 3D volume data.
    pub fn rebuild_pipeline_with_volume_data(&mut self, id: LayerId, data: VolumeData) {
        if data.values.len() > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
            let detail = format!(
                "Volume data buffer size ({} elements / {} bytes) exceeds maximum GPU storage buffer limit ({})",
                data.values.len(),
                data.values.len() * 4,
                crate::plots::common::MAX_GPU_STORAGE_BUFFER_BYTES
            );
            self.notify(
                crate::ui::toast::Severity::Error,
                "Volume too large for the GPU",
                detail,
            );
            return;
        }
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };

        let mut upload_later = false;
        if let Some(wgpu_render_state) = &self.wgpu_render_state {
            let encoding = layer.volume_encoding();
            let renderers = &mut layer.renderers;
            let same_dimensions = layer.data.volume.as_ref().is_some_and(|v| {
                v.width == data.width && v.height == data.height && v.depth == data.depth
            }) && renderers
                .volume
                .as_ref()
                .is_some_and(|r| r.encoding() == encoding);

            if same_dimensions && renderers.volume.is_some() && renderers.point_cloud.is_some() {
                // Uploaded before the next paint, to the renderer on screen.
                upload_later = true;
            } else {
                let volume_renderer = VolumeRenderer::new(
                    &wgpu_render_state.device,
                    &wgpu_render_state.queue,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                    encoding,
                );
                let point_cloud_renderer = PointCloudRenderer::new(
                    &wgpu_render_state.device,
                    wgpu_render_state.target_format,
                    &data.values,
                    data.width as u32,
                    data.height as u32,
                );
                renderers.volume = volume_renderer.map(Arc::new);
                renderers.point_cloud = Some(Arc::new(point_cloud_renderer));
                // New renderers start from `data`: nothing is pending.
                renderers.volume_dirty = None;
                renderers.point_cloud_dirty = None;
            }
        }

        let var_key = format!(
            "{}:{}:3d",
            layer.selection().store_target,
            layer.selection().variable_idx
        );
        let is_new_variable = layer.data.var_key.as_ref() != Some(&var_key);

        if is_new_variable {
            layer.clear_2d();
            layer.data.var_key = Some(var_key);
            layer.color.reset_to_extent(data.min_val, data.max_val);
        } else {
            layer.color.follow_extent(data.min_val, data.max_val);
        }

        let depth = data.depth;
        layer.data.volume = Some(data);
        layer.data.touch();
        if upload_later {
            layer.renderers.mark_volume_dirty(0..depth);
        }
    }
}

// ---------------------------------------------------------------------------
// Coastline LOD reload (asynchronous background fetch with non-blocking UI)
// ---------------------------------------------------------------------------

impl OctantApp {
    /// Dispatches an asynchronous fetch for the requested coastline LOD
    /// and swaps vertices into GPU buffers when ready without blocking UI frames.
    pub fn reload_coastline_lod(&mut self, lod: crate::plots::CoastlineLod) {
        if lod == self.coastline_current_lod || self.coastline_is_loading {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.coastline_rx = Some(rx);
        self.coastline_is_loading = true;
        crate::plots::fetch_coastline_async(lod, tx);
        log::info!("Coastline LOD fetch requested for {:?}", lod);
    }
}
