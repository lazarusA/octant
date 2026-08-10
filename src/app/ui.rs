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

                        let source_id =
                            format!("{:?}:{}", self.selected_store_kind, self.store_target_input);
                        let kind = match self.selected_store_kind {
                            crate::app::StoreKind::RemoteZarr => {
                                crate::data::DataSourceKind::RemoteZarr
                            }
                            crate::app::StoreKind::LocalZarr => {
                                crate::data::DataSourceKind::LocalZarr
                            }
                            crate::app::StoreKind::RemoteIcechunk => {
                                crate::data::DataSourceKind::RemoteIcechunk
                            }
                            crate::app::StoreKind::LocalIcechunk => {
                                crate::data::DataSourceKind::LocalIcechunk
                            }
                            crate::app::StoreKind::ProceduralRandom => {
                                crate::data::DataSourceKind::Other("ProceduralRandom".into())
                            }
                        };
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

            let is_busy = self
                .active_block_key
                .as_ref()
                .is_some_and(|key| self.block_prefetcher.is_pending(key));

            // Only advance playback when current requested slice is already loaded & rendered
            if !is_busy {
                if now.duration_since(self.last_step_time) >= frame_dur {
                    self.last_step_time = now;
                    let max_steps = self
                        .matrix_data
                        .as_ref()
                        .map(|h| h.max_timesteps)
                        .unwrap_or(1);

                    if max_steps > 1 {
                        if self.current_timestep + 1 < max_steps {
                            self.current_timestep += 1;
                        } else if self.loop_playback {
                            self.current_timestep = 0;
                        } else {
                            self.is_playing = false;
                        }
                        self.load_selected_variable_block();
                    }
                }
            } else {
                // Keep timer fresh while waiting for slice download
                self.last_step_time = now;
            }
            ctx.request_repaint_after(frame_dur);
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
        crate::ui::loading_bar::show_canvas_loading_bar(self, &ctx, canvas_rect);

        // 4. Drawing Canvas Area with Aspect Data Ratio
        {
            let canvas_rect = ui.available_rect_before_wrap();
            let response = ui.allocate_rect(canvas_rect, egui::Sense::drag());

            let canvas_bg = ui.style().visuals.panel_fill;
            ui.painter().rect_filled(canvas_rect, 0.0, canvas_bg);

            // Enforce aspect data ratio (matrix.width / matrix.height)
            let plot_rect = if let Some(matrix) = &self.matrix_data {
                let data_aspect = (matrix.width as f32 / matrix.height as f32).max(0.01);
                let avail_w = canvas_rect.width();
                let avail_h = canvas_rect.height();
                let avail_aspect = avail_w / avail_h.max(1.0);

                let (plot_w, plot_h) = if avail_aspect > data_aspect {
                    (avail_h * data_aspect, avail_h)
                } else {
                    (avail_w, avail_w / data_aspect)
                };

                egui::Rect::from_center_size(canvas_rect.center(), egui::vec2(plot_w, plot_h))
            } else {
                canvas_rect
            };

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
                    self.sphere_zoom = (self.sphere_zoom - scroll * 0.003).clamp(1.1, 8.0);
                    ui.ctx().request_repaint();
                }
            }

            if self.sphere_auto_rotate
                && (self.active_plot_type == PlotType::Sphere
                    || self.active_plot_type == PlotType::Surface
                    || self.active_plot_type == PlotType::Volume
                    || self.active_plot_type == PlotType::PointCloud)
            {
                self.sphere_rotation_y += ui.ctx().input(|i| i.stable_dt).min(0.1) * 0.15;
                ui.ctx().request_repaint();
            }

            match self.active_plot_type {
                PlotType::Line => {
                    if let Some(line_renderer) = &self.line_renderer {
                        let color_params = self.get_color_params();
                        let (profile_values, profile_length, line_count) =
                            self.get_line_profile_payload();
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            LineCallback {
                                renderer: line_renderer.clone(),
                                color_params,
                                rect: plot_rect,
                                profile_values,
                                profile_length,
                                line_count,
                                line_mode: if self.line_plot_all_series { 1 } else { 0 },
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
                            .matrix_data
                            .as_ref()
                            .map_or((64, 64), |m| (m.width as u32, m.height as u32));
                        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();

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
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                PlotType::PointCloud => {
                    if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                        let (width, height) = self
                            .matrix_data
                            .as_ref()
                            .map_or((64, 64), |m| (m.width as u32, m.height as u32));
                        let (aspect_x, aspect_y, aspect_z) = self.get_3d_aspect_ratio();

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
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
                _ => {
                    if let Some(renderer) = &self.renderer {
                        let callback = eframe::egui_wgpu::Callback::new_paint_callback(
                            plot_rect,
                            MatrixCallback {
                                renderer: renderer.clone(),
                                color_params: self.get_color_params(),
                                rect: plot_rect,
                            },
                        );
                        ui.painter().add(callback);
                    }
                }
            }

            // Render high-performance Hover Pixel Info Tooltip & Canvas Reticle
            crate::ui::hover_tooltip::show_hover_tooltip(self, &ctx, ui, &response, plot_rect);
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
