//! Canvas paint callback assembly and active 2D renderer buffer updates.

use std::sync::Arc;

use crate::app::OctantApp;
use crate::plots::PlotType;

#[derive(Clone, Copy)]
struct Common3DSpatialContext {
    width: u32,
    height: u32,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    shift_x: u32,
    shift_y: u32,
    shift_z: u32,
    color: crate::plots::PlotColorParams,
}

impl OctantApp {
    /// Updates GPU vertex/storage buffer data for the currently active 2D renderer.
    pub fn update_active_2d_renderer_data(&self, queue: &wgpu::Queue, values: &[f32]) {
        match self.active_plot_type {
            PlotType::Heatmap => {
                if let Some(renderer) = &self.renderer {
                    renderer.update_data(queue, values);
                }
            }
            PlotType::Sphere => {
                if let Some(sphere_renderer) = &self.sphere_renderer {
                    sphere_renderer.update_data(queue, values);
                }
            }
            PlotType::Surface | PlotType::Block => {
                if let Some(surface_renderer) = &self.surface_renderer {
                    surface_renderer.update_data(queue, values);
                }
            }
            PlotType::Line => {
                if let Some(line_renderer) = &self.line_renderer {
                    line_renderer.update_data(queue, values);
                }
            }
            PlotType::Volume | PlotType::PointCloud => {}
        }
    }

    /// Resolves active (width, height) for 3D Volume and PointCloud shaders.
    pub fn get_volume_dimensions(&self) -> (u32, u32) {
        self.volume_data
            .as_ref()
            .map(|v| (v.width as u32, v.height as u32))
            .unwrap_or_else(|| {
                self.matrix_data
                    .as_ref()
                    .map_or((64, 64), |m| (m.width as u32, m.height as u32))
            })
    }

    /// Extracts common 3D spatial dimensions, aspect ratios, shifts, and color uniforms.
    #[inline]
    fn get_common_3d_spatial_context(&self) -> Common3DSpatialContext {
        let (width, height) = self.get_volume_dimensions();
        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();
        let (shift_x, shift_y, shift_z) = self.get_volume_shifts();
        let color = self.get_color_params();

        Common3DSpatialContext {
            width,
            height,
            aspect_x,
            aspect_y,
            aspect_z,
            shift_x,
            shift_y,
            shift_z,
            color,
        }
    }

    /// Assembles VolumeUniformParams for 3D volume raymarching.
    pub fn get_volume_uniform_params(
        &self,
        screen_aspect: f32,
    ) -> crate::plots::VolumeUniformParams {
        let ctx = self.get_common_3d_spatial_context();

        crate::plots::VolumeUniformParams {
            color: ctx.color,
            rot_y: self.sphere_rotation_y,
            rot_x: self.sphere_rotation_x,
            aspect_x: ctx.aspect_x,
            aspect_y: ctx.aspect_y,
            aspect_z: ctx.aspect_z,
            zoom: self.sphere_zoom,
            opacity_scale: self.volume_opacity,
            step_count: self.volume_step_count,
            width: ctx.width,
            height: ctx.height,
            algorithm: self.volume_algorithm,
            isovalue: self.volume_isovalue,
            isorange: self.volume_isorange,
            attenuation: self.volume_attenuation,
            screen_aspect,
            shift_x: ctx.shift_x,
            shift_y: ctx.shift_y,
            shift_z: ctx.shift_z,
            transparency: self.volume_transparency,
        }
    }

    /// Assembles PointCloudUniformParams for 3D point cloud billboard rendering.
    pub fn get_point_cloud_uniform_params(
        &self,
        screen_aspect: f32,
    ) -> crate::plots::PointCloudUniformParams {
        let ctx = self.get_common_3d_spatial_context();

        crate::plots::PointCloudUniformParams {
            color: ctx.color,
            rot_y: self.sphere_rotation_y,
            rot_x: self.sphere_rotation_x,
            aspect_x: ctx.aspect_x,
            aspect_y: ctx.aspect_y,
            aspect_z: ctx.aspect_z,
            zoom: self.sphere_zoom,
            point_size: self.point_cloud_size,
            width: ctx.width,
            height: ctx.height,
            screen_aspect,
            shift_x: ctx.shift_x,
            shift_y: ctx.shift_y,
            shift_z: ctx.shift_z,
        }
    }

    /// Assembles Mesh3DUniformParams for Sphere and Surface heightfields (zero allocation).
    pub fn get_mesh_3d_uniform_params(
        &self,
        mode: u32,
        displacement_strength: f32,
        aspect_ratio: f32,
    ) -> crate::plots::Mesh3DUniformParams {
        let (coord_mode, has_reference_globe, lon_bounds, lat_bounds) = self
            .matrix_data
            .as_ref()
            .map(|m| {
                (
                    m.grid.render_coord_mode(),
                    !m.grid.is_global(),
                    m.grid.lon_bounds_rad(),
                    m.grid.lat_bounds_rad(),
                )
            })
            .unwrap_or((
                0,
                false,
                [-std::f32::consts::PI, std::f32::consts::PI],
                [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            ));

        crate::plots::Mesh3DUniformParams {
            color: self.get_color_params(),
            rotation_y: self.sphere_rotation_y,
            rotation_x: self.sphere_rotation_x,
            aspect_ratio,
            zoom: self.sphere_zoom,
            displacement_strength,
            mode,
            coord_mode,
            has_reference_globe,
            lon_bounds,
            lat_bounds,
        }
    }

    /// Polls the asynchronous coastline receiver and hot-swaps GPU buffers upon completion.
    fn poll_coastline_receiver(&mut self) {
        let Some(rx) = &self.coastline_rx else { return };
        if let Ok(result) = rx.try_recv() {
            self.coastline_rx = None;
            self.coastline_is_loading = false;
            match result {
                Ok((lod, verts)) => {
                    if let Some(wgpu_state) = &self.wgpu_render_state {
                        if let Some(r) = &self.coastline_renderer {
                            r.swap_vertices(&wgpu_state.device, &wgpu_state.queue, &verts);
                        }
                        if let Some(r3d) = &self.coastline_3d_renderer {
                            r3d.swap_vertices(&wgpu_state.device, &wgpu_state.queue, &verts);
                        }
                    }
                    self.coastline_current_lod = lod;
                    log::info!("Coastline hot-swapped to {:?}", lod);
                }
                Err(e) => {
                    log::warn!("Async coastline fetch failed: {e}");
                }
            }
        }
    }

    /// Dispatches the appropriate GPU paint callback to the egui painter for the active plot type.
    pub fn paint_active_plot(
        &mut self,
        ui: &mut egui::Ui,
        canvas_rect: egui::Rect,
        plot_rect: egui::Rect,
        gpu_pan: [f32; 2],
        gpu_zoom: f32,
        gpu_aspect_scale: [f32; 2],
    ) {
        self.poll_coastline_receiver();

        match self.active_plot_type {
            crate::plots::PlotType::Line => {
                if let Some(line_renderer) = &self.line_renderer {
                    let color_params = self.get_color_params();
                    let (profile_values, profile_length, line_count) =
                        self.get_line_profile_payload();
                    let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                        canvas_rect,
                        crate::plots::LineCallback {
                            renderer: line_renderer.clone(),
                            color_params,
                            rect: canvas_rect,
                            profile_values,
                            profile_length,
                            line_count,
                            line_mode: if self.line_plot_all_series { 1 } else { 0 },
                            pan: gpu_pan,
                            zoom: gpu_zoom,
                        },
                    );
                    ui.painter().add(callback);
                }
            }
            crate::plots::PlotType::Sphere => {
                if let Some(sphere_renderer) = &self.sphere_renderer {
                    let aspect_ratio = crate::plots::common::compute_aspect_ratio(&plot_rect);
                    let params = self.get_mesh_3d_uniform_params(
                        self.sphere_mode,
                        self.sphere_displacement_strength,
                        aspect_ratio,
                    );
                    let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                        plot_rect,
                        crate::plots::Mesh3DCallback {
                            renderer: sphere_renderer.clone(),
                            params,
                            cube_mode_idx: 3,
                            rect: plot_rect,
                        },
                    );
                    ui.painter().add(callback);
                }
            }
            crate::plots::PlotType::Surface | crate::plots::PlotType::Block => {
                if let Some(surface_renderer) = &self.surface_renderer {
                    let aspect_ratio = crate::plots::common::compute_aspect_ratio(&plot_rect);
                    let params = self.get_mesh_3d_uniform_params(
                        if self.active_plot_type == crate::plots::PlotType::Block {
                            2
                        } else {
                            self.surface_mode
                        },
                        self.surface_displacement_strength,
                        aspect_ratio,
                    );
                    let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                        plot_rect,
                        crate::plots::Mesh3DCallback {
                            renderer: surface_renderer.clone(),
                            params,
                            cube_mode_idx: 2,
                            rect: plot_rect,
                        },
                    );
                    ui.painter().add(callback);
                }
            }
            crate::plots::PlotType::Volume => {
                if let Some(volume_renderer) = &self.volume_renderer {
                    let screen_aspect = crate::plots::common::compute_aspect_ratio(&plot_rect);
                    let params = self.get_volume_uniform_params(screen_aspect);
                    let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                        plot_rect,
                        crate::plots::VolumeCallback {
                            renderer: volume_renderer.clone(),
                            params,
                            rect: plot_rect,
                        },
                    );
                    ui.painter().add(callback);
                }
            }
            crate::plots::PlotType::PointCloud => {
                if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                    let screen_aspect = crate::plots::common::compute_aspect_ratio(&plot_rect);
                    let params = self.get_point_cloud_uniform_params(screen_aspect);
                    let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                        plot_rect,
                        crate::plots::PointCloudCallback {
                            renderer: point_cloud_renderer.clone(),
                            params,
                            rect: plot_rect,
                        },
                    );
                    ui.painter().add(callback);
                }
            }
            _ => {
                if let Some(renderer) = &self.renderer {
                    if self.active_pyramid.is_some()
                        && self.active_plot_type == crate::plots::PlotType::Heatmap
                    {
                        let ((u_min, u_max), (v_min, v_max)) =
                            crate::data::ViewportResampler::compute_visible_data_bounds(
                                gpu_pan,
                                gpu_zoom,
                                gpu_aspect_scale,
                            );
                        let (orig_w, orig_h) = self.active_data_dimensions_2d();
                        let (target_w, target_h) =
                            crate::data::ViewportResampler::compute_target_resolution(
                                orig_w, orig_h, 2048,
                            );

                        if let Some(tile) = self.resampler.resample_if_needed(
                            (u_min, u_max),
                            (v_min, v_max),
                            target_w,
                            target_h,
                        ) && let Some(wgpu_render_state) = &self.wgpu_render_state
                        {
                            renderer.update_data_and_dimensions(
                                &wgpu_render_state.queue,
                                &tile.data.values,
                                tile.data.width,
                                tile.data.height,
                                tile.tile_bounds,
                            );
                        }
                    }

                    let coord_mode = self
                        .matrix_data
                        .as_ref()
                        .map_or(0, |m| m.grid.render_coord_mode());
                    let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                        canvas_rect,
                        crate::plots::MatrixCallback {
                            renderer: renderer.clone(),
                            color_params: self.get_color_params(),
                            rect: canvas_rect,
                            pan: gpu_pan,
                            zoom: gpu_zoom,
                            aspect_scale: gpu_aspect_scale,
                            coord_mode,
                        },
                    );
                    ui.painter().add(callback);
                }
            }
        }

        // --- Coastline overlay ---
        let coastline_supported = matches!(
            self.active_plot_type,
            PlotType::Heatmap | PlotType::Surface | PlotType::Block | PlotType::Sphere
        );
        if self.show_coastlines && coastline_supported {
            if let Some(cr) = self.coastline_renderer.as_ref().map(Arc::clone) {
                // Theme-aware default: white in dark mode, dark gray in light mode
                let line_color = self.coastline_color.unwrap_or_else(|| {
                    if ui.visuals().dark_mode {
                        [1.0, 1.0, 1.0, 0.75]
                    } else {
                        [0.15, 0.15, 0.15, 0.85]
                    }
                });

                // Extract dataset geographic bounds from the active grid so the
                // shader can project coastline lon/lat into the dataset's domain.
                let (lon_min, lon_max, lat_min, lat_max) = self
                    .matrix_data
                    .as_ref()
                    .map(|m| crate::plots::dataset_geo_bounds(&m.grid))
                    .unwrap_or((-180.0, 180.0, 90.0, -90.0));

                if self.active_plot_type == PlotType::Heatmap {
                    let cb = eframe::egui_wgpu::Callback::new_paint_callback(
                        canvas_rect,
                        crate::plots::CoastlineCallback {
                            renderer: cr,
                            pan: gpu_pan,
                            zoom: gpu_zoom,
                            crop_to_domain: self.coastline_crop_to_data_domain,
                            aspect_scale: gpu_aspect_scale,
                            line_color,
                            rect: canvas_rect,
                            lon_min,
                            lon_max,
                            lat_min,
                            lat_max,
                        },
                    );
                    ui.painter().add(cb);
                }
            }

            if self.active_plot_type != PlotType::Heatmap
                && let Some(renderer) = self.coastline_3d_renderer.as_ref().map(Arc::clone)
            {
                let (mode, plot_kind, displacement_strength) = match self.active_plot_type {
                    PlotType::Sphere => (self.sphere_mode, 1, self.sphere_displacement_strength),
                    PlotType::Block => (2, 0, self.surface_displacement_strength),
                    _ => (self.surface_mode, 0, self.surface_displacement_strength),
                };
                let mesh_params = self.get_mesh_3d_uniform_params(
                    mode,
                    displacement_strength,
                    crate::plots::common::compute_aspect_ratio(&plot_rect),
                );
                let line_color = self.coastline_color.unwrap_or_else(|| {
                    if ui.visuals().dark_mode {
                        [1.0, 1.0, 1.0, 0.75]
                    } else {
                        [0.15, 0.15, 0.15, 0.85]
                    }
                });
                let cb = eframe::egui_wgpu::Callback::new_paint_callback(
                    plot_rect,
                    crate::plots::Coastline3DCallback {
                        renderer,
                        params: crate::plots::Coastline3DParams {
                            rotation_y: mesh_params.rotation_y,
                            rotation_x: mesh_params.rotation_x,
                            aspect_ratio: mesh_params.aspect_ratio,
                            zoom: mesh_params.zoom,
                            displacement_strength: mesh_params.displacement_strength,
                            plot_kind,
                            plot_mode: mode,
                            coord_mode: mesh_params.coord_mode,
                            crop_to_domain: if self.coastline_crop_to_data_domain {
                                1
                            } else {
                                0
                            },
                            lon_bounds: mesh_params.lon_bounds,
                            lat_bounds: mesh_params.lat_bounds,
                            color: line_color,
                            color_range: [mesh_params.color.cmin, mesh_params.color.cmax],
                        },
                        rect: plot_rect,
                    },
                );
                ui.painter().add(cb);
            }
        }
    }
}
