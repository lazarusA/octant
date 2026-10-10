use crate::ui::toast::Severity;

use super::OctantApp;

impl eframe::App for OctantApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_colormap_prefs(storage);
    }

    /// Only colormap preferences are persisted; window and widget state are not.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Reset hover preview at start of frame
        self.preview_colormap = None;

        // Notices reported by renderers, backends and background threads.
        self.drain_reported_notices();

        // 0. Poll completed background metadata inspection
        let mut metadata_done = false;
        if let Some(rx) = &self.metadata_rx {
            if let Ok(result) = rx.try_recv() {
                metadata_done = true;
                self.is_loading = false;
                match result {
                    Ok((metadata, store_handle)) => {
                        if metadata.variables.is_empty() {
                            self.status_message =
                                format!("No variables discovered in '{}'", metadata.name);
                            self.notify(Severity::Warning, "No variables found", &metadata.name);
                        } else {
                            self.status_message = format!(
                                "Inspected '{}' (Found {} variables)",
                                metadata.name,
                                metadata.variables.len()
                            );
                        }
                        self.layout.hero_state.loading = false;
                        self.layout.hero_state.loaded = true;
                        self.layout.hero_state.source_label = metadata.name.clone();
                        self.layout.show_variables_overlay = true;

                        let source_id = self.selected_source_id();
                        let mut dataset = crate::data::Dataset::new(
                            &source_id,
                            store_handle.source().clone(),
                            store_handle,
                        );
                        dataset.metadata = Some(metadata.clone());
                        self.dataset_manager.add(dataset);

                        self.load_new_metadata(metadata);
                    }
                    Err(err) => {
                        self.layout.hero_state.loading = false;
                        self.layout.hero_state.loaded = false;
                        self.clear_active_metadata();
                        self.status_message = format!("Store inspect error: {}", err);
                        self.notify(Severity::Error, "Couldn't open dataset", &err);
                    }
                }
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
        if metadata_done {
            self.metadata_rx = None;
        }

        // 1. Drain completed block-cache prefetch results and arrived coordinates.
        self.poll_block_prefetch_results();
        self.poll_coordinate_results();

        // 2. Playback Animation Timer Loop
        let is_minimized = ctx.input(|i| {
            i.viewport().minimized.unwrap_or(false) || i.viewport().occluded.unwrap_or(false)
        });

        if self.playback.is_playing && !is_minimized {
            let now = web_time::Instant::now();
            let frame_dur =
                std::time::Duration::from_secs_f32(1.0 / self.playback.playback_fps.max(1.0));

            if now.duration_since(self.playback.last_step_time) >= frame_dur {
                self.advance_playback(now);
            }

            let elapsed = now.duration_since(self.playback.last_step_time);
            let next_wake = if elapsed < frame_dur {
                frame_dur - elapsed
            } else {
                std::time::Duration::from_millis(1)
            };
            ctx.request_repaint_after(next_wake);
        } else if self.playback.is_playing && is_minimized {
            // When minimized or occluded, poll infrequently (500ms) without advancing playback or hammering the GPU.
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        } else if self.block_prefetcher.pending_count() > 0
            || self.coordinate_loader.pending_count() > 0
            || self.metadata_rx.is_some()
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        let is_hero_active = (self.layers.base.data.matrix.is_none()
            && self.layers.base.data.volume.is_none())
            || self.layout.show_hero;

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

        if !ctx.egui_wants_keyboard_input()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::C))
        {
            self.show_crop_overlay = !self.show_crop_overlay;
        }

        // Process in-flight export / screenshot requests
        self.process_pending_export(&ctx);

        // Global Drag-and-Drop handler for files and directories
        let dropped_file = ctx.input(|i| i.raw.dropped_files.first().cloned());
        if let Some(file) = dropped_file {
            let path = file.path();
            let path_str = path.to_string_lossy().trim().to_string();
            if !path_str.is_empty() {
                match crate::utils::infer_store_kind_from_target(&path_str) {
                    Ok(_) => {
                        crate::ui::drop_zone::clear_drop_zone_warning(&ctx);
                        self.layout.hero_state.input = path_str.clone();
                        self.selected.store_target = path_str.clone();
                        self.submit_or_activate_source(&path_str, None);
                    }
                    Err(err) => {
                        self.status_message = format!("{err}: '{path_str}'");
                        crate::ui::drop_zone::trigger_drop_zone_warning(&ctx);
                        self.notify(
                            Severity::Warning,
                            "Unsupported file",
                            format!("{err}: {path_str}"),
                        );
                    }
                }
            }
        }

        // 3. Render panels (each consumes space from the remaining area)
        crate::ui::top_bar::show_top_bar(self, ui);

        if self.layout.show_left_panel {
            crate::ui::store::show_left_panel(self, ui);
        }

        if !is_hero_active && self.has_animated_dimension() {
            crate::ui::bottom_bar::show_bottom_bar(self, ui);
        }

        crate::ui::catalog::show_catalog_window(self, &ctx);
        crate::ui::about::show_about_window(self, &ctx);
        crate::ui::about::show_icon_gallery_window(self, &ctx);
        crate::ui::export_modal::show_export_modal(self, &ctx);

        // Overlays anchor relative to the remaining canvas rect
        let canvas_rect = ui.available_rect_before_wrap();
        self.show_floating_panels(&ctx, canvas_rect, !is_hero_active);
        crate::ui::toast::show_toasts(self, &ctx, canvas_rect);

        // 4. Drawing Canvas Area with Aspect Data Ratio
        super::canvas::render_canvas(self, ui, canvas_rect);
    }
}
