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
        match self.effective_canvas_plot_type() {
            PlotType::Heatmap => {
                if let Some(renderer) = &self.layers.base.renderers.heatmap {
                    renderer.update_data(queue, values);
                }
            }
            PlotType::Sphere => {
                if let Some(sphere_renderer) = &self.layers.base.renderers.sphere {
                    sphere_renderer.update_data(queue, values);
                }
            }
            PlotType::Surface => {
                if let Some(surface_renderer) = &self.layers.base.renderers.surface {
                    surface_renderer.update_data(queue, values);
                }
            }
            PlotType::Line => {
                if let Some(line_renderer) = &self.layers.base.renderers.line {
                    line_renderer.update_data(queue, values);
                }
            }
            PlotType::Volume | PlotType::PointCloud => {}
        }
    }

    /// Resolves active (width, height) for 3D Volume and PointCloud shaders.
    pub fn get_volume_dimensions(&self) -> (u32, u32) {
        self.layers
            .base
            .data
            .volume
            .as_ref()
            .map(|v| (v.width as u32, v.height as u32))
            .unwrap_or_else(|| {
                self.layers
                    .base
                    .data
                    .matrix
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
        let mut ctx = self.get_common_3d_spatial_context();
        // Classic modes (1-7) keep their original opacity.
        if self.volume_algorithm != 0 {
            ctx.color.opacity = 1.0;
            ctx.color.alpha_row = crate::utils::colormap::NO_ALPHA_ROW;
        }

        crate::plots::VolumeUniformParams {
            color: ctx.color,
            rot_y: self.sphere_rotation_y,
            rot_x: self.sphere_rotation_x,
            aspect_x: ctx.aspect_x,
            aspect_y: ctx.aspect_y,
            aspect_z: ctx.aspect_z,
            zoom: self.sphere_zoom,
            opacity_scale: self.volume_opacity,
            quality: self.volume_quality,
            algorithm: self.volume_algorithm,
            isovalue: self.volume_isovalue,
            isorange: self.volume_isorange,
            attenuation: self.volume_attenuation,
            screen_aspect,
            shift_x: ctx.shift_x,
            shift_y: ctx.shift_y,
            shift_z: ctx.shift_z,
            transparency: self.volume_transparency,
            lighting: self.volume_lighting,
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
            .layers
            .base
            .data
            .matrix
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
                    self.notify(
                        crate::ui::toast::Severity::Warning,
                        "Coastlines unavailable",
                        &e.to_string(),
                    );
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
        let canvas_plot_type = self.effective_canvas_plot_type();
        self.release_idle_oit_frames(canvas_plot_type);

        let view = super::paint_layer::CanvasView {
            canvas_rect,
            plot_rect,
            pan: gpu_pan,
            zoom: gpu_zoom,
            aspect_scale: gpu_aspect_scale,
        };
        self.refresh_resampled_tiles(canvas_plot_type, &view);
        for layer in self.layers.iter() {
            self.paint_layer(ui, layer, canvas_plot_type, &view);
        }

        // --- Coastline overlay ---
        let coastline_supported = matches!(
            canvas_plot_type,
            PlotType::Heatmap | PlotType::Surface | PlotType::Sphere
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
                    .layers
                    .base
                    .data
                    .matrix
                    .as_ref()
                    .map(|m| crate::plots::dataset_geo_bounds(&m.grid))
                    .unwrap_or((-180.0, 180.0, 90.0, -90.0));

                if canvas_plot_type == PlotType::Heatmap {
                    let cb = eframe::egui_wgpu::Callback::new_paint_callback(
                        canvas_rect,
                        crate::plots::CoastlineCallback {
                            renderer: cr,
                            pan: gpu_pan,
                            zoom: gpu_zoom,
                            crop_to_domain: self.coastline_crop_to_data_domain,
                            aspect_scale: gpu_aspect_scale,
                            line_color,
                            line_width: self.coastline_line_width,
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

            if canvas_plot_type != PlotType::Heatmap
                && let Some(renderer) = self.coastline_3d_renderer.as_ref().map(Arc::clone)
            {
                let (mode, plot_kind, displacement_strength) = match canvas_plot_type {
                    PlotType::Sphere => (self.sphere_mode, 1, self.sphere_displacement_strength),
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
                            line_width: self.coastline_line_width,
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
