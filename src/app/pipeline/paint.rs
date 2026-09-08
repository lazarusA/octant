//! Canvas paint callback assembly and active 2D renderer buffer updates.

use crate::app::OctantApp;
use crate::plots::PlotType;

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

    /// Assembles VolumeUniformParams for 3D volume raymarching.
    pub fn get_volume_uniform_params(
        &self,
        screen_aspect: f32,
    ) -> crate::plots::VolumeUniformParams {
        let (width, height) = self.get_volume_dimensions();
        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();
        let (shift_x, shift_y, shift_z) = self.get_volume_shifts();

        crate::plots::VolumeUniformParams {
            color: self.get_color_params(),
            rot_y: self.sphere_rotation_y,
            rot_x: self.sphere_rotation_x,
            aspect_x,
            aspect_y,
            aspect_z,
            zoom: self.sphere_zoom,
            opacity_scale: self.volume_opacity,
            step_count: self.volume_step_count,
            width,
            height,
            algorithm: self.volume_algorithm,
            isovalue: self.volume_isovalue,
            isorange: self.volume_isorange,
            attenuation: self.volume_attenuation,
            screen_aspect,
            shift_x,
            shift_y,
            shift_z,
            transparency: self.volume_transparency,
        }
    }

    /// Assembles PointCloudUniformParams for 3D point cloud billboard rendering.
    pub fn get_point_cloud_uniform_params(
        &self,
        screen_aspect: f32,
    ) -> crate::plots::PointCloudUniformParams {
        let (width, height) = self.get_volume_dimensions();
        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();
        let (shift_x, shift_y, shift_z) = self.get_volume_shifts();

        crate::plots::PointCloudUniformParams {
            color: self.get_color_params(),
            rot_y: self.sphere_rotation_y,
            rot_x: self.sphere_rotation_x,
            aspect_x,
            aspect_y,
            aspect_z,
            zoom: self.sphere_zoom,
            point_size: self.point_cloud_size,
            width,
            height,
            screen_aspect,
            shift_x,
            shift_y,
            shift_z,
        }
    }

    /// Assembles Mesh3DUniformParams for Sphere and Surface heightfields.
    pub fn get_mesh_3d_uniform_params(
        &self,
        mode: u32,
        displacement_strength: f32,
        aspect_ratio: f32,
    ) -> crate::plots::Mesh3DUniformParams {
        let grid = self
            .matrix_data
            .as_ref()
            .map(|m| m.grid.clone())
            .unwrap_or_default();
        let has_reference_globe = !grid.is_global();

        crate::plots::Mesh3DUniformParams {
            color: self.get_color_params(),
            rotation_y: self.sphere_rotation_y,
            rotation_x: self.sphere_rotation_x,
            aspect_ratio,
            zoom: self.sphere_zoom,
            displacement_strength,
            mode,
            grid,
            has_reference_globe,
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
            crate::plots::PlotType::Surface => {
                if let Some(surface_renderer) = &self.surface_renderer {
                    let aspect_ratio = crate::plots::common::compute_aspect_ratio(&plot_rect);
                    let params = self.get_mesh_3d_uniform_params(
                        self.surface_mode,
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

                    let coord_mode = self.matrix_data.as_ref().map_or(0, |m| m.grid.coord_mode());
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
    }
}
