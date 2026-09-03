use crate::plots::PlotType;
use crate::utils::apply_zoom_pan_at_point;

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
                        self.hero_state.loading = false;
                        self.hero_state.loaded = true;
                        self.hero_state.source_label = metadata.name.clone();
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
                        self.show_variables_overlay = true;
                    }
                    Err(err) => {
                        self.hero_state.loading = false;
                        self.status_message = format!("Store inspect error: {}", err);
                    }
                }
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
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
                let total_extent = self.animated_dim_extent();

                if total_extent > 1 {
                    let next_ts = if self.current_timestep + 1 < total_extent {
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
                        let base_request = self.plotted_variable_info().map(|v| {
                            crate::ui::variables_panel::build_slice_request_for_plotted(
                                self, &v.name, &v.shape,
                            )
                        });
                        let selections = base_request
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

                            // Continuous lookahead streaming: keep forward chunks queued in parallel ahead of playhead
                            if let Some(meta) = &self.plotted_dataset_metadata
                                && let Some(var) = meta.variables.get(self.plotted_variable_idx)
                            {
                                let shape = var.shape.clone();
                                self.prefetch_selected_animated_range(&shape);
                            }
                        } else {
                            // Target block window for next_ts is not yet in cache.
                            // Hold on current valid frame and trigger parallel lookahead prefetch without hijacking playhead.
                            if let Some(meta) = &self.plotted_dataset_metadata
                                && let Some(var) = meta.variables.get(self.plotted_variable_idx)
                            {
                                let shape = var.shape.clone();
                                self.prefetch_selected_animated_range(&shape);
                            }
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
        } else if self.block_prefetcher.pending_count() > 0 || self.metadata_rx.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        let is_hero_active =
            (self.matrix_data.is_none() && self.volume_data.is_none()) || self.show_hero;

        // Keyboard Shortcuts for Figure Export & Crop Tool
        if ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::S)
        }) {
            self.quick_save_canvas();
        }

        if ui.input_mut(|i| {
            i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            ) || i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S)
        }) {
            self.show_export_modal = true;
        }

        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::C)) {
            self.show_crop_overlay = !self.show_crop_overlay;
        }

        // Process in-flight export / screenshot requests
        self.process_pending_export(&ctx);

        // 3. Render panels (each consumes space from the remaining area)ng area)
        crate::ui::top_bar::show_top_bar(self, ui);

        if self.show_left_panel {
            crate::ui::store::show_left_panel(self, ui);
        }

        if !is_hero_active {
            if self.show_bottom_bar {
                crate::ui::bottom_bar::show_bottom_bar(self, ui);
            }
            if self.show_colorbar {
                crate::ui::colorbar::show_colorbar_overlay(self, &ctx);
            }
        }

        crate::ui::catalog::show_catalog_window(self, &ctx);
        crate::ui::about::show_about_window(self, &ctx);
        crate::ui::export_modal::show_export_modal(self, &ctx);

        // Overlays anchor relative to the remaining canvas rect
        let canvas_rect = ui.available_rect_before_wrap();
        crate::ui::variables::show_variables_overlay(self, &ctx, canvas_rect);
        crate::ui::settings::show_settings_window(self, &ctx, canvas_rect);
        crate::ui::variables_panel::show_variable_controls(self, &ctx, canvas_rect);

        // 4. Drawing Canvas Area with Aspect Data Ratio
        {
            let canvas_rect = ui.available_rect_before_wrap();

            if let Some(ref mut req) = self.pending_export
                && req.canvas_rect_in_points == egui::Rect::NOTHING
            {
                req.canvas_rect_in_points = canvas_rect;
                req.pixels_per_point = ctx.pixels_per_point();
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                ctx.request_repaint();
            }

            // When no dataset is plotted or hero landing view is active, render the clean hero page
            if is_hero_active {
                let canvas_bg = ui.visuals().panel_fill;
                ui.painter().rect_filled(canvas_rect, 0.0, canvas_bg);
                crate::ui::hero::show_hero_landing(self, ui);
                return;
            }

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
                        let mouse_pos = response.hover_pos().unwrap_or(canvas_rect.center());
                        let center = canvas_rect.center();

                        match self.active_plot_type {
                            PlotType::Heatmap => {
                                let (zoom, pan) = apply_zoom_pan_at_point(
                                    self.heatmap_zoom,
                                    self.heatmap_pan,
                                    mouse_pos,
                                    center,
                                    scroll,
                                    0.1,
                                    50.0,
                                );
                                self.heatmap_zoom = zoom;
                                self.heatmap_pan = pan;
                            }
                            PlotType::Line => {
                                let (zoom, pan) = apply_zoom_pan_at_point(
                                    self.line_zoom,
                                    self.line_pan,
                                    mouse_pos,
                                    center,
                                    scroll,
                                    0.1,
                                    50.0,
                                );
                                self.line_zoom = zoom;
                                self.line_pan = pan;
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
            let gpu_aspect_scale = self.compute_aspect_scale(canvas_rect.size());
            let (transformed_plot_rect, gpu_pan, gpu_zoom) = if is_3d_canvas_plot {
                (canvas_rect, [0.0, 0.0], 1.0)
            } else if self.active_plot_type == PlotType::Line {
                let zoom = self.line_zoom;
                let pan = self.line_pan;
                let scaled_size = canvas_rect.size() * zoom;
                let scaled_center = canvas_rect.center() + pan;
                let rect = egui::Rect::from_center_size(scaled_center, scaled_size);
                let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
                let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));
                (rect, [gpu_pan_x, gpu_pan_y], zoom)
            } else {
                let [aspect_scale_x, aspect_scale_y] = gpu_aspect_scale;
                let zoom = self.heatmap_zoom;
                let pan = self.heatmap_pan;
                let plot_w = canvas_rect.width() * aspect_scale_x * zoom;
                let plot_h = canvas_rect.height() * aspect_scale_y * zoom;
                let scaled_center = canvas_rect.center() + pan;
                let rect = egui::Rect::from_center_size(scaled_center, egui::vec2(plot_w, plot_h));

                let gpu_pan_x = pan.x / (0.5 * canvas_rect.width().max(1.0));
                let gpu_pan_y = -pan.y / (0.5 * canvas_rect.height().max(1.0));
                (rect, [gpu_pan_x, gpu_pan_y], zoom)
            };

            let plot_rect = transformed_plot_rect;

            // Dispatch active plot GPU rendering callback
            self.paint_active_plot(
                ui,
                canvas_rect,
                plot_rect,
                gpu_pan,
                gpu_zoom,
                gpu_aspect_scale,
            );

            // Draw Dynamic Plot Axis Lines, Ticks, and Axis Titles
            if !is_3d_canvas_plot && let Some(matrix) = &self.matrix_data {
                let (x_dom, y_dom, x_label, y_label) = if self.active_plot_type == PlotType::Line {
                    let y_min = self.color_range_min as f64;
                    let y_max = self.color_range_max as f64;
                    let profile_len = match self.line_profile_dim_idx {
                        2 => self.volume_data.as_ref().map_or(matrix.width, |v| v.depth),
                        1 => matrix.height,
                        _ => matrix.width,
                    };

                    let y_name = self
                        .plotted_variable_info()
                        .map(|var| {
                            if let Some(u) = &var.units {
                                format!("{} [{u}]", var.name)
                            } else if let Some(u) = var.attributes.get("units") {
                                format!("{} [{u}]", var.name)
                            } else {
                                var.name.clone()
                            }
                        })
                        .unwrap_or_else(|| "Data Value".to_string());

                    let target_dim_idx = self.get_spatial_dim_index(self.line_profile_dim_idx);
                    let fallback_dim_name = match self.line_profile_dim_idx {
                        2 => "z",
                        1 => "y",
                        _ => "x",
                    };

                    let (x_bounds, x_title) = self.resolve_axis_bounds_and_title(
                        target_dim_idx,
                        fallback_dim_name,
                        profile_len,
                    );

                    (x_bounds, (y_min, y_max), x_title, y_name)
                } else {
                    let (orig_w, orig_h) = self.active_data_dimensions_2d();
                    let x_dim = self.get_spatial_dim_index(0);
                    let y_dim = self.get_spatial_dim_index(1);

                    let (x_bounds, x_title) =
                        self.resolve_axis_bounds_and_title(x_dim, "X", orig_w);
                    let (y_bounds, y_title) =
                        self.resolve_axis_bounds_and_title(y_dim, "Y", orig_h);

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

            // Render high-performance Hover Pixel Info Tooltip & Canvas Reticle (suppressed during export capture)
            if self.pending_export.is_none() {
                crate::ui::hover_tooltip::show_hover_tooltip(
                    self,
                    &ctx,
                    ui,
                    &response,
                    canvas_rect,
                );
            }

            // Camera subtle capture flash overlay (confined to ROI guides if active, shown after capture)
            if let Some(flash_start) = self.export_flash_timer {
                let elapsed = flash_start.elapsed().as_secs_f32();
                let duration = 0.32;
                if elapsed < duration {
                    let progress = (elapsed / duration).clamp(0.0, 1.0);
                    let alpha = ((1.0 - progress) * 120.0) as u8;
                    let flash_rect = if self.show_crop_overlay {
                        egui::Rect::from_min_max(
                            egui::pos2(
                                canvas_rect.left() + self.roi_crop_box.u_min * canvas_rect.width(),
                                canvas_rect.top() + self.roi_crop_box.v_min * canvas_rect.height(),
                            ),
                            egui::pos2(
                                canvas_rect.left() + self.roi_crop_box.u_max * canvas_rect.width(),
                                canvas_rect.top() + self.roi_crop_box.v_max * canvas_rect.height(),
                            ),
                        )
                    } else {
                        canvas_rect
                    };

                    ui.painter().rect_filled(
                        flash_rect,
                        0.0,
                        egui::Color32::from_white_alpha(alpha),
                    );
                    ui.painter().rect_stroke(
                        flash_rect,
                        0.0,
                        egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_unmultiplied(
                                100,
                                220,
                                255,
                                ((alpha as f32) * 1.5).min(255.0) as u8,
                            ),
                        ),
                        egui::StrokeKind::Inside,
                    );
                    ctx.request_repaint();
                } else {
                    self.export_flash_timer = None;
                }
            }

            // Render interactive Region of Interest (ROI) Guiding Lines & Crop Tool (suppressed during export capture)
            if self.show_crop_overlay
                && self.pending_export.is_none()
                && let Some(crate::ui::crop_overlay::CropOverlayAction::Save) =
                    crate::ui::crop_overlay::show_crop_overlay(
                        ui,
                        canvas_rect,
                        &mut self.roi_crop_box,
                        &mut self.show_crop_overlay,
                    )
            {
                self.quick_save_canvas();
            }

            // Render floating Save Figure Toast with "Reveal in Finder / Folder" action
            crate::ui::export_modal::show_export_toast(self, &ctx, canvas_rect);
        }
    }
}

impl OctantApp {
    /// Processes in-flight export and screenshot events dispatched by the frame lifecycle.
    fn process_pending_export(&mut self, ctx: &egui::Context) {
        let Some(req) = self.pending_export.take() else {
            return;
        };

        if req.canvas_rect_in_points == egui::Rect::NOTHING {
            self.pending_export = Some(req);
            return;
        }

        let screenshot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });

        let Some(image) = screenshot else {
            self.pending_export = Some(req);
            ctx.request_repaint();
            return;
        };

        self.export_flash_timer = Some(std::time::Instant::now());
        let (crop_x, crop_y, crop_w, crop_h) =
            req.compute_crop_rect(image.width() as u32, image.height() as u32);
        let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();

        let (cropped_rgba, final_w, final_h) = crate::export::crop_rgba_buffer(
            &rgba,
            image.width() as u32,
            image.height() as u32,
            crop_x,
            crop_y,
            crop_w,
            crop_h,
        );

        let var_name = self
            .plotted_variable_info()
            .map(|v| v.name.as_str())
            .unwrap_or("plot");
        let title = format!("Octant - {}", var_name);

        match crate::export::encode_figure(
            &cropped_rgba,
            final_w,
            final_h,
            req.format,
            req.jpeg_quality,
            &title,
            var_name,
        ) {
            Ok(data) => {
                if req.copy_to_clipboard {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let img_data = arboard::ImageData {
                                width: final_w as usize,
                                height: final_h as usize,
                                bytes: std::borrow::Cow::Borrowed(&cropped_rgba),
                            };
                            let _ = clipboard.set_image(img_data);
                        }
                    }
                    self.status_message = "✓ Copied figure to clipboard".to_string();
                } else if let Some(ref path) = req.output_path {
                    if let Err(e) = crate::export::save_exported_file(&data, path) {
                        self.status_message = format!("Export error: {}", e);
                    } else {
                        let filename = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "figure".to_string());
                        self.status_message = format!("✓ Saved figure to {}", path.display());
                        self.export_toast = Some(crate::export::ExportToastNotification {
                            file_path: path.clone(),
                            filename,
                            timestamp: std::time::Instant::now(),
                        });
                    }
                }
            }
            Err(err) => {
                self.status_message = format!("Encoding error: {}", err);
            }
        }
    }

    /// Opens the Settings panel and closes Store, Variables, Controls, and Catalog.
    pub fn open_only_settings_panel(&mut self) {
        self.show_settings_panel = true;
        self.show_left_panel = false;
        self.show_variables_overlay = false;
        self.show_variable_controls = false;
        self.show_catalog_window = false;
        self.show_about_window = false;
    }
}
