use crate::plots::{
    LineCallback, MatrixCallback, PlotType, PointCloudCallback, SphereCallback, SurfaceCallback,
    VolumeCallback,
};

use super::OctantApp;

impl eframe::App for OctantApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Reset hover preview at start of frame
        self.preview_colormap = None;

        // 0. Poll completed background metadata inspection
        let mut metadata_done = false;
        if let Some(rx) = &self.metadata_rx {
            if let Ok(result) = rx.try_recv() {
                metadata_done = true;
                self.is_loading = false;
                match result {
                    Ok(metadata) => {
                        self.status_message = format!(
                            "Inspected '{}' (Found {} variables)",
                            metadata.name,
                            metadata.variables.len()
                        );
                        self.show_variables_overlay = true;

                        let source_id = self.selected_source_id();
                        let kind = self.selected_store_kind.to_data_source_kind();
                        let data_source = crate::data::DataSource::new(
                            &source_id,
                            kind,
                            &self.store_target_input,
                            &metadata.name,
                        );
                        if let Ok(store) = crate::data::SourceFactory::open(data_source.clone()) {
                            let mut dataset =
                                crate::data::Dataset::new(&source_id, data_source, store);
                            dataset.metadata = Some(metadata.clone());
                            self.dataset_manager.add(dataset);
                        }

                        self.active_dataset_metadata = Some(metadata);
                        self.selected_variable_idx = 0;
                    }
                    Err(err) => {
                        self.status_message = format!("Store inspect error: {}", err);
                    }
                }
            } else {
                ctx.request_repaint();
            }
        }
        if metadata_done {
            self.metadata_rx = None;
        }

        // 1. Drain completed block-cache prefetch results.
        self.poll_block_prefetch_results();

        // 2. Playback Animation Timer Loop
        if self.is_playing {
            let now = std::time::Instant::now();
            let frame_dur = std::time::Duration::from_secs_f32(1.0 / self.playback_fps.max(1.0));

            if now.duration_since(self.last_step_time) >= frame_dur {
                let total_steps = self.animated_dim_extent();

                if total_steps > 0 && self.current_timestep >= total_steps {
                    self.current_timestep = total_steps - 1;
                }

                if total_steps > 1 {
                    let next_ts = if self.current_timestep + 1 < total_steps {
                        Some(self.current_timestep + 1)
                    } else if self.loop_playback {
                        Some(0)
                    } else {
                        self.is_playing = false;
                        None
                    };

                    if let Some(next_ts) = next_ts {
                        let source_id = self.plotted_source_id();
                        let var_name = self.plotted_variable_info().map(|v| v.name.clone());
                        let legacy_request = self.plotted_variable_info().map(|v| {
                            crate::ui::variables_panel::build_slice_request_for_plotted(
                                self, &v.name, &v.shape,
                            )
                        });
                        let selections = legacy_request
                            .as_ref()
                            .map(|r| r.selections.as_slice())
                            .unwrap_or(&[]);

                        let has_next_block = if let Some(ref name) = var_name {
                            self.block_cache.covers(
                                &source_id,
                                name,
                                selections,
                                self.plotted_animated_dim,
                                next_ts,
                            )
                        } else {
                            false
                        };

                        if has_next_block {
                            self.current_timestep = next_ts;
                            self.last_step_time = now;
                            self.load_selected_variable_block();

                            // Lookahead prefetch: stream upcoming chunks in the background while playing
                            if let Some(anim_dim) = self.plotted_animated_dim {
                                let anim_chunk_size = self.plotted_chunk_size(anim_dim).max(1);
                                let ahead_ts = next_ts + anim_chunk_size;
                                if ahead_ts < total_steps {
                                    self.prefetch_block_window_for_next_steps(ahead_ts);
                                }
                            }
                        } else {
                            // Target block window for next_ts is not yet in cache.
                            // Trigger background prefetch for next_ts block window while safely keeping playback on current valid frame.
                            self.prefetch_block_window_for_next_steps(next_ts);
                            self.last_step_time = now;
                        }
                    }
                } else {
                    self.is_playing = false;
                }
            }

            let elapsed = now.duration_since(self.last_step_time);
            let next_wake = if elapsed < frame_dur {
                frame_dur - elapsed
            } else {
                std::time::Duration::from_millis(1)
            };
            ctx.request_repaint_after(next_wake);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // 3. Render panels (each consumes space from the remaining area)
        crate::ui::top_bar::show_top_bar(self, ui);
        crate::ui::store::show_left_panel(self, ui);
        crate::ui::bottom_bar::show_bottom_bar(self, ui);
        crate::ui::catalog::show_catalog_window(self, &ctx);
        crate::ui::colorbar::show_colorbar_overlay(self, &ctx);

        // After panels consume their space, the remaining rect is the canvas.
        // Pass it to the overlays so they can anchor to the canvas left edge.
        let canvas_rect = ui.available_rect_before_wrap();
        crate::ui::variables::show_variables_overlay(self, &ctx, canvas_rect);
        crate::ui::settings::show_settings_window(self, &ctx, canvas_rect);
        crate::ui::variables_panel::show_variable_controls(self, &ctx, canvas_rect);

        // 4. Drawing Canvas Area with Aspect Data Ratio
        {
            let canvas_rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(canvas_rect, egui::Sense::drag());

            let canvas_bg = ui.style().visuals.panel_fill;
            ui.painter().rect_filled(canvas_rect, 0.0, canvas_bg);

            // 3D plots expand to full container width & height, using shader aspect projection to maintain 3D proportions
            let is_3d_canvas_plot = self.active_plot_type == PlotType::Sphere
                || self.active_plot_type == PlotType::Surface
                || self.active_plot_type == PlotType::Volume
                || self.active_plot_type == PlotType::PointCloud;

            // Handle Zoom & Pan Interactions
            if is_3d_canvas_plot {
                if response.double_clicked() {
                    self.sphere_rotation_x = 0.25;
                    self.sphere_rotation_y = 0.0;
                    self.sphere_zoom = 2.5;
                    ui.ctx().request_repaint();
                }

                if response.dragged() {
                    let delta = response.drag_delta();
                    self.sphere_rotation_y += delta.x * 0.008;
                    self.sphere_rotation_x = (self.sphere_rotation_x + delta.y * 0.008).clamp(
                        -std::f32::consts::FRAC_PI_2 + 0.05,
                        std::f32::consts::FRAC_PI_2 - 0.05,
                    );
                }

                if response.hovered() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll != 0.0 {
                        let min_zoom = if self.active_plot_type == PlotType::Sphere {
                            1.1
                        } else {
                            0.2
                        };
                        self.sphere_zoom = (self.sphere_zoom - scroll * 0.003).clamp(min_zoom, 8.0);
                        ui.ctx().request_repaint();
                    }
                }
            } else {
                // 2D Flatmap Heatmap & 1D Line Plot zoom & pan interaction
                if response.double_clicked() {
                    match self.active_plot_type {
                        PlotType::Heatmap => self.reset_heatmap_view(),
                        PlotType::Line => self.reset_line_view(),
                        _ => {}
                    }
                }

                if response.dragged() {
                    let delta = response.drag_delta();
                    match self.active_plot_type {
                        PlotType::Heatmap => self.heatmap_pan += delta,
                        PlotType::Line => self.line_pan += delta,
                        _ => {}
                    }
                }

                if response.hovered() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll != 0.0 {
                        let zoom_factor = (1.0 + scroll * 0.002).clamp(0.8, 1.25);
                        let mouse_pos = response.hover_pos().unwrap_or(canvas_rect.center());
                        let center = canvas_rect.center();

                        match self.active_plot_type {
                            PlotType::Heatmap => {
                                let old_zoom = self.heatmap_zoom;
                                let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 50.0);
                                let old_pan = self.heatmap_pan;
                                self.heatmap_pan = old_pan * (new_zoom / old_zoom)
                                    + (mouse_pos - center) * (1.0 - new_zoom / old_zoom);
                                self.heatmap_zoom = new_zoom;
                            }
                            PlotType::Line => {
                                let old_zoom = self.line_zoom;
                                let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 50.0);
                                let old_pan = self.line_pan;
                                self.line_pan = old_pan * (new_zoom / old_zoom)
                                    + (mouse_pos - center) * (1.0 - new_zoom / old_zoom);
                                self.line_zoom = new_zoom;
                            }
                            _ => {}
                        }
                        ui.ctx().request_repaint();
                    }
                }
            }

            if self.sphere_auto_rotate && is_3d_canvas_plot {
                self.sphere_rotation_y += ui.ctx().input(|i| i.stable_dt).min(0.1) * 0.15;
                ui.ctx().request_repaint();
            }

            // Compute screen-space transformed plot rect and GPU pan/zoom uniforms
            let (transformed_plot_rect, gpu_pan, gpu_zoom, gpu_aspect_scale) = if is_3d_canvas_plot
            {
                (canvas_rect, [0.0, 0.0], 1.0, [1.0, 1.0])
            } else if self.active_plot_type == PlotType::Line {
                let zoom = self.line_zoom;
                let pan = self.line_pan;
                let scaled_size = canvas_rect.size() * zoom;
                let scaled_center = canvas_rect.center() + pan;
                let rect = egui::Rect::from_center_size(scaled_center, scaled_size);
                let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
                let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));
                (rect, [gpu_pan_x, gpu_pan_y], zoom, [1.0, 1.0])
            } else {
                let (aspect_scale_x, aspect_scale_y) = if self.enforce_data_aspect_ratio
                    && let Some(matrix) = &self.matrix_data
                {
                    let data_aspect = (matrix.width as f32 / matrix.height as f32).max(0.001);
                    let canvas_aspect = canvas_rect.width() / canvas_rect.height().max(1.0);
                    if canvas_aspect > data_aspect {
                        (data_aspect / canvas_aspect, 1.0)
                    } else {
                        (1.0, canvas_aspect / data_aspect)
                    }
                } else {
                    (1.0, 1.0)
                };

                let zoom = self.heatmap_zoom;
                let pan = self.heatmap_pan;
                let plot_w = canvas_rect.width() * aspect_scale_x * zoom;
                let plot_h = canvas_rect.height() * aspect_scale_y * zoom;
                let scaled_center = canvas_rect.center() + pan;
                let rect = egui::Rect::from_center_size(scaled_center, egui::vec2(plot_w, plot_h));

                let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
                let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));
                (
                    rect,
                    [gpu_pan_x, gpu_pan_y],
                    zoom,
                    [aspect_scale_x, aspect_scale_y],
                )
            };

            let plot_rect = transformed_plot_rect;

            match self.active_plot_type {
                PlotType::Line => {
                    if let Some(line_renderer) = &self.line_renderer {
                        let color_params = self.get_color_params();
                        let (profile_values, profile_length, line_count) =
                            self.get_line_profile_payload();
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            canvas_rect,
                            LineCallback {
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
                PlotType::Sphere => {
                    if let Some(sphere_renderer) = &self.sphere_renderer {
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            SphereCallback {
                                renderer: sphere_renderer.clone(),
                                color_params: self.get_color_params(),
                                rotation_y: self.sphere_rotation_y,
                                rotation_x: self.sphere_rotation_x,
                                zoom: self.sphere_zoom,
                                displacement_strength: self.sphere_displacement_strength,
                                sphere_mode: self.sphere_mode,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::Surface => {
                    if let Some(surface_renderer) = &self.surface_renderer {
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            SurfaceCallback {
                                renderer: surface_renderer.clone(),
                                color_params: self.get_color_params(),
                                rotation_y: self.sphere_rotation_y,
                                rotation_x: self.sphere_rotation_x,
                                zoom: self.sphere_zoom,
                                displacement_strength: self.surface_displacement_strength,
                                surface_mode: self.surface_mode,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::Volume => {
                    if let Some(volume_renderer) = &self.volume_renderer {
                        let (width, height) = self
                            .volume_data
                            .as_ref()
                            .map(|v| (v.width as u32, v.height as u32))
                            .unwrap_or_else(|| {
                                self.matrix_data
                                    .as_ref()
                                    .map_or((64, 64), |m| (m.width as u32, m.height as u32))
                            });
                        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();
                        let (shift_x, shift_y, shift_z) = self.get_volume_shifts();

                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            VolumeCallback {
                                renderer: volume_renderer.clone(),
                                color_params: self.get_color_params(),
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
                                shift_x,
                                shift_y,
                                shift_z,
                                transparency: self.volume_transparency,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::PointCloud => {
                    if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                        let (width, height) = self
                            .volume_data
                            .as_ref()
                            .map(|v| (v.width as u32, v.height as u32))
                            .unwrap_or_else(|| {
                                self.matrix_data
                                    .as_ref()
                                    .map_or((64, 64), |m| (m.width as u32, m.height as u32))
                            });
                        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();
                        let (shift_x, shift_y, shift_z) = self.get_volume_shifts();

                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            PointCloudCallback {
                                renderer: point_cloud_renderer.clone(),
                                color_params: self.get_color_params(),
                                rot_y: self.sphere_rotation_y,
                                rot_x: self.sphere_rotation_x,
                                aspect_x,
                                aspect_y,
                                aspect_z,
                                zoom: self.sphere_zoom,
                                point_size: self.point_cloud_size,
                                width,
                                height,
                                shift_x,
                                shift_y,
                                shift_z,
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                _ => {
                    if let Some(renderer) = &self.renderer {
                        if self.active_pyramid.is_some() && self.active_plot_type == PlotType::Heatmap {
                            let ((u_min, u_max), (v_min, v_max)) =
                                crate::data::ViewportResampler::compute_visible_data_bounds(
                                    gpu_pan,
                                    gpu_zoom,
                                    gpu_aspect_scale,
                                );
                            let max_res = 2048;
                            let target_w = self.matrix_data.as_ref().map_or(1024, |m| m.width.min(max_res));
                            let target_h = self.matrix_data.as_ref().map_or(1024, |m| m.height.min(max_res));

                            if let Some(sampled) = self.resampler.resample_if_needed(
                                (u_min, u_max),
                                (v_min, v_max),
                                target_w,
                                target_h,
                            ) && let Some(wgpu_render_state) = &self.wgpu_render_state {
                                renderer.update_data_and_dimensions(
                                    &wgpu_render_state.queue,
                                    &sampled.values,
                                    sampled.width,
                                    sampled.height,
                                );
                            }
                        }

                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            canvas_rect,
                            MatrixCallback {
                                renderer: renderer.clone(),
                                color_params: self.get_color_params(),
                                rect: canvas_rect,
                                pan: gpu_pan,
                                zoom: gpu_zoom,
                                aspect_scale: gpu_aspect_scale,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
            }

            // Draw Dynamic Plot Axis Lines, Ticks, and Axis Titles
            if !is_3d_canvas_plot && let Some(matrix) = &self.matrix_data {
                let (x_dom, y_dom, x_label, y_label) = if self.active_plot_type == PlotType::Line {
                    let y_min = self.color_range_min as f64;
                    let y_max = self.color_range_max as f64;
                    let profile_len = match self.line_profile_dim_idx {
                        2 => self.volume_data.as_ref().map_or(matrix.width, |v| v.depth) as f64,
                        1 => matrix.height as f64,
                        _ => matrix.width as f64,
                    };

                    let mut x_bounds = (0.0, (profile_len - 1.0).max(1.0));
                    let mut x_name = match self.line_profile_dim_idx {
                        2 => "z".to_string(),
                        1 => "y".to_string(),
                        _ => "x".to_string(),
                    };
                    let mut y_name = "Data Value".to_string();

                    if let Some(meta) = &self.plotted_dataset_metadata
                        && let Some(var) = meta.variables.get(self.plotted_variable_idx)
                    {
                        if let Some(u) = &var.units {
                            y_name = format!("{} [{u}]", var.name);
                        } else if let Some(u) = var.attributes.get("units") {
                            y_name = format!("{} [{u}]", var.name);
                        } else {
                            y_name = var.name.clone();
                        }

                        let (x_dim, y_dim, z_dim) = Self::resolve_spatial_axes(
                            var.shape.len(),
                            &var.dimension_names,
                            &var.dimension_names,
                            &self.plotted_dim_config,
                        );

                        let target_dim_idx = match self.line_profile_dim_idx {
                            2 => z_dim,
                            1 => y_dim,
                            _ => x_dim,
                        };

                        if target_dim_idx < var.shape.len() {
                            let dim_size = var
                                .shape
                                .get(target_dim_idx)
                                .copied()
                                .unwrap_or(profile_len as u64)
                                as usize;
                            let (start_p, end_p) = self
                                .plotted_selected_dim_ranges
                                .get(target_dim_idx)
                                .copied()
                                .unwrap_or((0, dim_size.saturating_sub(1)));
                            x_bounds = (start_p as f64, end_p as f64);

                            if let Some(name) = var.dimension_names.get(target_dim_idx) {
                                x_name = name.clone();
                                if let Some(bounds) = meta.get_coord_bounds_for_range(
                                    name,
                                    dim_size,
                                    (start_p, end_p),
                                ) {
                                    x_bounds = bounds;
                                }
                            }
                        }
                    }

                    let x_title = crate::utils::coordinates::format_dimension_axis_title(&x_name);

                    (x_bounds, (y_min, y_max), x_title, y_name)
                } else {
                    let mut x_name = "X".to_string();
                    let mut y_name = "Y".to_string();
                    let mut x_bounds = (0.0, matrix.width.saturating_sub(1) as f64);
                    let mut y_bounds = (0.0, matrix.height.saturating_sub(1) as f64);

                    if let Some(meta) = &self.plotted_dataset_metadata
                        && let Some(var) = meta.variables.get(self.plotted_variable_idx)
                    {
                        let (x_dim, y_dim, _) = Self::resolve_spatial_axes(
                            var.shape.len(),
                            &var.dimension_names,
                            &var.dimension_names,
                            &self.plotted_dim_config,
                        );

                        if x_dim < var.shape.len() {
                            let dim_size =
                                var.shape.get(x_dim).copied().unwrap_or(matrix.width as u64)
                                    as usize;
                            let (start_x, end_x) = self
                                .plotted_selected_dim_ranges
                                .get(x_dim)
                                .copied()
                                .unwrap_or((0, dim_size.saturating_sub(1)));
                            x_bounds = (start_x as f64, end_x as f64);

                            if let Some(x_n) = var.dimension_names.get(x_dim) {
                                x_name = x_n.clone();
                                if let Some(bounds) =
                                    meta.get_coord_bounds_for_range(x_n, dim_size, (start_x, end_x))
                                {
                                    x_bounds = bounds;
                                }
                            }
                        }

                        if y_dim < var.shape.len() {
                            let dim_size = var
                                .shape
                                .get(y_dim)
                                .copied()
                                .unwrap_or(matrix.height as u64)
                                as usize;
                            let (start_y, end_y) = self
                                .plotted_selected_dim_ranges
                                .get(y_dim)
                                .copied()
                                .unwrap_or((0, dim_size.saturating_sub(1)));
                            y_bounds = (start_y as f64, end_y as f64);

                            if let Some(y_n) = var.dimension_names.get(y_dim) {
                                y_name = y_n.clone();
                                if let Some(bounds) =
                                    meta.get_coord_bounds_for_range(y_n, dim_size, (start_y, end_y))
                                {
                                    y_bounds = bounds;
                                }
                            }
                        }
                    } else {
                        if let Some(&(start_x, end_x)) = self.plotted_selected_dim_ranges.first() {
                            x_bounds = (start_x as f64, end_x as f64);
                        }
                        if let Some(&(start_y, end_y)) = self.plotted_selected_dim_ranges.get(1) {
                            y_bounds = (start_y as f64, end_y as f64);
                        }
                    }

                    let x_title = crate::utils::coordinates::format_dimension_axis_title(&x_name);
                    let y_title = crate::utils::coordinates::format_dimension_axis_title(&y_name);

                    (x_bounds, y_bounds, x_title, y_title)
                };

                let options = crate::ui::axes::PlotAxisOptions {
                    x_domain: x_dom,
                    y_domain: y_dom,
                    x_title: &x_label,
                    y_title: &y_label,
                };

                crate::ui::axes::draw_plot_axes(ui, canvas_rect, plot_rect, &options);
            }

            // Render high-performance Hover Pixel Info Tooltip & Canvas Reticle
            crate::ui::hover_tooltip::show_hover_tooltip(self, &ctx, ui, &response, canvas_rect);
        }
    }
}

impl OctantApp {
    /// Opens the Settings panel and closes Store, Variables, Controls, and Catalog.
    pub fn open_only_settings_panel(&mut self) {
        self.show_settings_panel = true;
        self.show_left_panel = false;
        self.show_variables_overlay = false;
        self.show_variable_controls = false;
        self.show_catalog_window = false;
    }
}
