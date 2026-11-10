//! Canvas paint callback assembly and active 2D renderer buffer updates.

use std::sync::Arc;

use crate::app::OctantApp;
use crate::app::layers::Layer;
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
    /// Resolves `layer`'s (width, height) for 3D Volume and PointCloud shaders.
    pub fn get_volume_dimensions(layer: &Layer) -> (u32, u32) {
        layer
            .data
            .volume
            .as_ref()
            .map(|v| (v.width as u32, v.height as u32))
            .unwrap_or_else(|| {
                layer
                    .data
                    .matrix
                    .as_ref()
                    .map_or((64, 64), |m| (m.width as u32, m.height as u32))
            })
    }

    /// Extracts `layer`'s 3D dimensions and color uniforms, with the canvas's
    /// aspect ratios and shifts.
    #[inline]
    fn get_common_3d_spatial_context(&self, layer: &Layer) -> Common3DSpatialContext {
        let (width, height) = Self::get_volume_dimensions(layer);
        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();
        let (shift_x, shift_y, shift_z) = self.get_volume_shifts();
        let color = self.get_color_params(layer);

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

    /// Assembles `layer`'s VolumeUniformParams for 3D volume raymarching.
    pub fn get_volume_uniform_params(
        &self,
        layer: &Layer,
        screen_aspect: f32,
    ) -> crate::plots::VolumeUniformParams {
        let mut ctx = self.get_common_3d_spatial_context(layer);
        // Classic modes (1-7) keep their original opacity; label surfaces
        // take the layer's with Transparency on.
        if !matches!(self.plot_configs.volume.algorithm, 0 | 4) {
            ctx.color.opacity = 1.0;
            ctx.color.alpha_row = crate::utils::colormap::NO_ALPHA_ROW;
        }

        crate::plots::VolumeUniformParams {
            color: ctx.color,
            camera: self
                .nav
                .orbit_camera(PlotType::Volume, screen_aspect)
                .uniform(),
            // The unit box's y runs up and z toward the viewer.
            slice: self
                .plot_configs
                .slice
                .uniform_in(crate::plots::slice::VOLUME_FLIP, None),
            aspect_x: ctx.aspect_x,
            aspect_y: ctx.aspect_y,
            aspect_z: ctx.aspect_z,
            opacity_scale: self.plot_configs.volume.opacity,
            quality: self.plot_configs.volume.quality,
            algorithm: self.plot_configs.volume.algorithm,
            surface: self.plot_configs.volume.surface as u32,
            isovalue: self.plot_configs.volume.isovalue_in(layer.color.extent()),
            attenuation: self.plot_configs.volume.attenuation,
            shift_x: ctx.shift_x,
            shift_y: ctx.shift_y,
            shift_z: ctx.shift_z,
            transparency: self.plot_configs.volume.transparency,
            lighting: self.plot_configs.volume.lighting,
        }
    }

    /// Assembles `layer`'s PointCloudUniformParams for 3D point cloud billboard rendering.
    pub fn get_point_cloud_uniform_params(
        &self,
        layer: &Layer,
        screen_aspect: f32,
    ) -> crate::plots::PointCloudUniformParams {
        let ctx = self.get_common_3d_spatial_context(layer);

        crate::plots::PointCloudUniformParams {
            color: ctx.color,
            camera: self
                .nav
                .orbit_camera(PlotType::PointCloud, screen_aspect)
                .uniform(),
            slice: self.plot_configs.slice.uniform(),
            aspect_x: ctx.aspect_x,
            aspect_y: ctx.aspect_y,
            aspect_z: ctx.aspect_z,
            point_size: self.plot_configs.point_cloud.point_size,
            scale_by_value: self.plot_configs.point_cloud.scale_by_value,
            // A thinned cloud's own grid (`PointGrid`).
            width: layer.data.points.map_or(ctx.width, |g| g.width as u32),
            height: layer.data.points.map_or(ctx.height, |g| g.height as u32),
            shift_x: ctx.shift_x,
            shift_y: ctx.shift_y,
            shift_z: ctx.shift_z,
        }
    }

    /// Assembles `layer`'s Mesh3DUniformParams for Sphere and Surface heightfields (zero allocation).
    pub fn get_mesh_3d_uniform_params(
        &self,
        layer: &Layer,
        sphere: bool,
        aspect_ratio: f32,
    ) -> crate::plots::Mesh3DUniformParams {
        let (mode, displacement_strength, flip_height) = self.plot_configs.mesh.heights(sphere);
        let plot_type = if sphere {
            PlotType::Sphere
        } else {
            PlotType::Surface
        };
        let (coord_mode, lon_bounds, lat_bounds) = layer
            .data
            .matrix
            .as_ref()
            .map(|m| {
                let mode = match sphere {
                    true => m.grid.sphere_coord_mode(),
                    false => m.grid.render_coord_mode(),
                };
                (mode, m.grid.lon_bounds_rad(), m.grid.lat_bounds_rad())
            })
            .unwrap_or((
                0,
                [-std::f32::consts::PI, std::f32::consts::PI],
                [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2],
            ));

        crate::plots::Mesh3DUniformParams {
            color: self.get_color_params(layer),
            camera: self.nav.orbit_camera(plot_type, aspect_ratio).uniform(),
            slice: self.mesh_slice(layer, sphere, (mode, displacement_strength, flip_height)),
            displacement_strength,
            mode,
            coord_mode,
            lon_bounds,
            lat_bounds,
            flip_height,
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
                        crate::i18n::tr(crate::i18n::NoticesKey::CoastlinesUnavailable),
                        e.to_string(),
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
        let now = ui.input(|i| i.time);
        if let Some(wait) = self.refresh_detail(canvas_plot_type, &view, now) {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(wait.max(0.0)));
        }
        for layer in self.layers.iter().filter(|l| l.is_drawn()) {
            self.paint_layer(ui, layer, canvas_plot_type, &view);
        }

        // --- Coastline overlay ---
        // Coastlines only over geographic data (the setting stays on for when
        // a geographic plot returns).
        if self.draws_coastlines(canvas_plot_type) {
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
                let (lon_min, lon_max, lat_min, lat_max) = self.coastline_bounds_2d();

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
                            // Cut with the heatmap only when the user asks.
                            slice: self
                                .plot_configs
                                .slice
                                .for_coastlines(self.plot_configs.slice.uniform()),
                        },
                    );
                    ui.painter().add(cb);
                }
            }

            self.paint_coastlines_3d(ui, plot_rect, canvas_plot_type);
        }
    }
}
