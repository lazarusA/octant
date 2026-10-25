use crate::i18n::{NoticesKey as N, tr};
use crate::ui::toast::Severity;

use super::OctantApp;

impl eframe::App for OctantApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.save_language_pref(storage);
        self.save_colormap_prefs(storage);
    }

    /// Only colormap preferences are persisted; window and widget state are not.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.preview_colormap = None;

        // Notices reported by renderers, backends and background threads.
        self.drain_reported_notices();
        crate::ui::fonts::FontEngine::watch_dataset(&ctx, self);
        super::window_title::sync(&ctx);

        // 0. Poll completed background metadata inspection
        self.poll_inspection(&ctx);
        self.poll_plate(&ctx);

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

        let is_hero_active = self.hero_shown();

        crate::ui::keyboard::dispatch(self, &ctx, !is_hero_active);

        // Process in-flight export / screenshot requests
        self.process_pending_export();

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
                            tr(N::UnsupportedFile),
                            format!("{err}: {path_str}"),
                        );
                    }
                }
            }
        }

        // 3. Bars, modal windows and floating panels around the canvas
        let canvas_rect = self.show_chrome(ui, !is_hero_active);
        crate::ui::toast::show_toasts(self, &ctx, canvas_rect);

        // 4. Drawing Canvas Area with Aspect Data Ratio
        super::canvas::render_canvas(self, ui, canvas_rect);
        crate::ui::keyboard::focus::end_frame(&ctx);
    }
}
