use crate::app::OctantApp;

/// Draws interactive canvas framing guides, shaded letterbox scrim, shutter flash, and save toast notifications.
pub fn draw_canvas_framing_guides(app: &mut OctantApp, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
    let data_aspect = if let Some(m) = &app.matrix_data {
        if m.height > 0 {
            Some(m.width as f32 / m.height as f32)
        } else {
            None
        }
    } else {
        None
    };

    let capture_rect = app
        .capture_config
        .compute_capture_rect(canvas_rect, data_aspect);

    // 1. Shutter Flash Effect on Capture
    if let Some(flash_t) = app.capture_config.shutter_flash_time {
        let elapsed = flash_t.elapsed().as_secs_f32();
        if elapsed < 0.25 {
            let alpha = ((1.0 - elapsed / 0.25) * 180.0) as u8;
            ui.painter().rect_filled(
                capture_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
            );
            ui.ctx().request_repaint();
        } else {
            app.capture_config.shutter_flash_time = None;
        }
    }

    // 2. Framing Guides & Scrim
    if app.capture_config.show_framing_guides || app.capture_config.is_recording {
        let scrim_color = egui::Color32::from_black_alpha(130);

        // Shaded Letterbox Scrim outside the capture area
        if capture_rect.top() > canvas_rect.top() {
            let top_scrim = egui::Rect::from_min_max(
                canvas_rect.min,
                egui::pos2(canvas_rect.max.x, capture_rect.top()),
            );
            ui.painter().rect_filled(top_scrim, 0.0, scrim_color);
        }
        if capture_rect.bottom() < canvas_rect.bottom() {
            let bottom_scrim = egui::Rect::from_min_max(
                egui::pos2(canvas_rect.min.x, capture_rect.bottom()),
                canvas_rect.max,
            );
            ui.painter().rect_filled(bottom_scrim, 0.0, scrim_color);
        }
        if capture_rect.left() > canvas_rect.left() {
            let left_scrim = egui::Rect::from_min_max(
                egui::pos2(canvas_rect.min.x, capture_rect.top()),
                egui::pos2(capture_rect.left(), capture_rect.bottom()),
            );
            ui.painter().rect_filled(left_scrim, 0.0, scrim_color);
        }
        if capture_rect.right() < canvas_rect.right() {
            let right_scrim = egui::Rect::from_min_max(
                egui::pos2(capture_rect.right(), capture_rect.top()),
                egui::pos2(canvas_rect.max.x, capture_rect.bottom()),
            );
            ui.painter().rect_filled(right_scrim, 0.0, scrim_color);
        }

        // Framing Bounding Border & Rule-of-Thirds Grid (Rule-of-thirds preview only shown when not actively recording)
        let border_stroke = egui::Stroke::new(1.5, egui::Color32::from_white_alpha(180));
        ui.painter()
            .rect_stroke(capture_rect, 2.0, border_stroke, egui::StrokeKind::Outside);

        let is_recording = app.capture_config.is_recording;

        if !is_recording {
            let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(35));
            let third_w = capture_rect.width() / 3.0;
            let third_h = capture_rect.height() / 3.0;

            // Vertical rule-of-thirds lines
            ui.painter().line_segment(
                [
                    egui::pos2(capture_rect.left() + third_w, capture_rect.top()),
                    egui::pos2(capture_rect.left() + third_w, capture_rect.bottom()),
                ],
                grid_stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(capture_rect.left() + third_w * 2.0, capture_rect.top()),
                    egui::pos2(capture_rect.left() + third_w * 2.0, capture_rect.bottom()),
                ],
                grid_stroke,
            );

            // Horizontal rule-of-thirds lines
            ui.painter().line_segment(
                [
                    egui::pos2(capture_rect.left(), capture_rect.top() + third_h),
                    egui::pos2(capture_rect.right(), capture_rect.top() + third_h),
                ],
                grid_stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(capture_rect.left(), capture_rect.top() + third_h * 2.0),
                    egui::pos2(capture_rect.right(), capture_rect.top() + third_h * 2.0),
                ],
                grid_stroke,
            );
        }

        // Corner brackets pointing outward into the scrim area
        let corner_len = 14.0;
        let corner_stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(0, 180, 255));
        let corners = [
            (
                capture_rect.left_top(),
                egui::vec2(-1.0, 0.0),
                egui::vec2(0.0, -1.0),
            ),
            (
                capture_rect.right_top(),
                egui::vec2(1.0, 0.0),
                egui::vec2(0.0, -1.0),
            ),
            (
                capture_rect.left_bottom(),
                egui::vec2(-1.0, 0.0),
                egui::vec2(0.0, 1.0),
            ),
            (
                capture_rect.right_bottom(),
                egui::vec2(1.0, 0.0),
                egui::vec2(0.0, 1.0),
            ),
        ];
        for (pos, dir_x, dir_y) in corners {
            ui.painter()
                .line_segment([pos, pos + dir_x * corner_len], corner_stroke);
            ui.painter()
                .line_segment([pos, pos + dir_y * corner_len], corner_stroke);
        }

        // Header Badge Pill shown only in preview mode (never during active recording)
        if !is_recording {
            let ppp = ui.ctx().pixels_per_point();
            let phys_w = (capture_rect.width() * ppp).round() as u32;
            let phys_h = (capture_rect.height() * ppp).round() as u32;

            let badge_y = if capture_rect.top() - canvas_rect.top() >= 32.0 {
                capture_rect.top() - 16.0
            } else if canvas_rect.bottom() - capture_rect.bottom() >= 32.0 {
                capture_rect.bottom() + 16.0
            } else {
                canvas_rect.top() + 16.0
            };
            let badge_pos = egui::pos2(capture_rect.center().x, badge_y);

            let badge_rect = egui::Rect::from_center_size(badge_pos, egui::vec2(180.0, 26.0));

            ui.painter().rect_filled(
                badge_rect,
                13.0,
                egui::Color32::from_rgba_premultiplied(20, 20, 25, 220),
            );
            ui.painter().rect_stroke(
                badge_rect,
                13.0,
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(70)),
                egui::StrokeKind::Outside,
            );

            let mut badge_ui = ui.new_child(egui::UiBuilder::new().max_rect(badge_rect));
            badge_ui.horizontal_centered(|ui| {
                ui.add_space(8.0);
                ui.small(
                    egui::RichText::new(format!(
                        "📐 {} ({}×{})",
                        app.capture_config.aspect_preset.short_name(),
                        phys_w,
                        phys_h
                    ))
                    .strong(),
                );
            });
        }
    }

    // 3. Floating Toast Notification for Saved Plots & Recordings
    let mut clear_notification = false;
    if let Some((title, path, created_t)) = &app.capture_config.save_notification {
        let elapsed = created_t.elapsed().as_secs_f32();
        if elapsed < 4.5 {
            ui.ctx().request_repaint();
            let toast_w = 340.0;
            let toast_h = 58.0;
            let toast_rect = egui::Rect::from_min_size(
                egui::pos2(
                    canvas_rect.right() - toast_w - 18.0,
                    canvas_rect.top() + 18.0,
                ),
                egui::vec2(toast_w, toast_h),
            );

            let bg_color = egui::Color32::from_rgba_premultiplied(16, 22, 30, 245);
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(46, 204, 113));
            ui.painter().rect_filled(toast_rect, 8.0, bg_color);
            ui.painter()
                .rect_stroke(toast_rect, 8.0, border_stroke, egui::StrokeKind::Outside);

            let mut toast_ui = ui.new_child(egui::UiBuilder::new().max_rect(toast_rect));
            toast_ui.horizontal_centered(|ui| {
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .small()
                                .strong()
                                .color(egui::Color32::from_rgb(46, 204, 113)),
                        );
                    });
                    let path_display = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                    ui.label(
                        egui::RichText::new(format!("📁 ~/Downloads/{}", path_display))
                            .small()
                            .weak(),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if ui
                        .button("✕")
                        .on_hover_text("Dismiss notification")
                        .clicked()
                    {
                        clear_notification = true;
                    }

                    let reveal_btn =
                        egui::Button::new(egui::RichText::new("Reveal").small().strong());
                    if ui
                        .add(reveal_btn)
                        .on_hover_text("Show file in Finder / file manager")
                        .clicked()
                    {
                        reveal_in_finder(path);
                    }
                });
            });
        } else {
            clear_notification = true;
        }
    }

    if clear_notification {
        app.capture_config.save_notification = None;
    }

    show_export_progress_modal(app, ui.ctx());
}

/// Displays an on-screen progress modal while deterministic animation export is active.
pub fn show_export_progress_modal(app: &mut OctantApp, ctx: &egui::Context) {
    let mut cancel_requested = false;
    if let Some(ref export) = app.capture_config.export_state
        && export.is_active
    {
        egui::Window::new("🎬 Exporting Animation Video")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(300.0);
                let current = export.current_frame;
                let total = export.total_frames.max(1);
                let progress = (current as f32) / (total as f32);

                ui.add(egui::ProgressBar::new(progress).show_percentage());
                ui.add_space(4.0);
                ui.label(format!("Rendering frame {} of {}...", current, total));
                ui.label(
                    egui::RichText::new(format!(
                        "Mode: {} | Zoom: {}",
                        export.motion_mode.label(),
                        export.zoom_mode.label()
                    ))
                    .small()
                    .weak(),
                );
                ui.separator();
                if ui.button("❌ Cancel Export").clicked() {
                    cancel_requested = true;
                }
            });
    }

    if cancel_requested {
        app.cancel_deterministic_export();
    }
}

/// Reveals a file in macOS Finder, Windows Explorer, or Linux file manager.
#[allow(unused_variables)]
pub fn reveal_in_finder(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,\"{}\"", path.display()))
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }
}
