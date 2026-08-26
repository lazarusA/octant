use crate::app::OctantApp;
use crate::export::{ExportFormat, ExportTarget};
use std::path::PathBuf;

/// Shows the floating modal dialog for saving and exporting the canvas/figure.
pub fn show_export_modal(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_export_modal {
        return;
    }

    let mut is_open = app.show_export_modal;
    let mut should_close = false;

    egui::Window::new("📸 Save & Export Figure")
        .open(&mut is_open)
        .resizable(false)
        .collapsible(false)
        .default_width(380.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            // 1. Format Selection Tabs
            ui.label(egui::RichText::new("Export Format").strong());
            ui.horizontal_wrapped(|ui| {
                for format in ExportFormat::ALL {
                    let is_selected = app.export_settings.format == format;
                    if ui.selectable_label(is_selected, format.label()).clicked() {
                        app.export_settings.format = format;
                    }
                }
            });

            if app.export_settings.format == ExportFormat::Jpeg {
                ui.horizontal(|ui| {
                    ui.label("JPEG Quality:");
                    ui.add(
                        egui::Slider::new(&mut app.export_settings.jpeg_quality, 10..=100)
                            .suffix("%"),
                    );
                });
            }

            ui.separator();

            // 2. Export Target (Full Canvas vs ROI)
            ui.label(egui::RichText::new("Export Area").strong());
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut app.export_settings.target,
                    ExportTarget::FullCanvas,
                    "🖼️ Full Canvas (with Overlays)",
                );
                ui.radio_value(
                    &mut app.export_settings.target,
                    ExportTarget::RoiCrop,
                    "✂️ Framed ROI",
                );
            });

            if app.export_settings.target == ExportTarget::RoiCrop {
                ui.horizontal(|ui| {
                    if ui
                        .button(if app.show_crop_overlay {
                            "Hide Guiding Lines"
                        } else {
                            "Show Guiding Lines on Canvas"
                        })
                        .clicked()
                    {
                        app.show_crop_overlay = !app.show_crop_overlay;
                    }
                });
            }

            ui.separator();

            // 3. Export Directory & Filename Preview
            ui.label(egui::RichText::new("Destination & Filename").strong());
            ui.horizontal(|ui| {
                ui.label("Folder:");
                ui.text_edit_singleline(&mut app.export_settings.export_dir);
            });

            let var_name = app
                .plotted_variable_info()
                .map(|v| v.name.as_str())
                .unwrap_or("plot");
            let default_name = generate_export_filename(var_name, app.export_settings.format);

            ui.label(
                egui::RichText::new(format!(
                    "Preview: {}/{}",
                    app.export_settings.export_dir, default_name
                ))
                .small()
                .weak(),
            );

            ui.separator();

            // 4. Action Buttons
            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("💾 Save Figure").strong())
                    .on_hover_text("Save figure to disk (Cmd+S)")
                    .clicked()
                {
                    let out_path = crate::export::resolve_export_path(
                        &app.export_settings.export_dir,
                        &default_name,
                    );
                    app.request_canvas_export(out_path, false);
                    should_close = true;
                }

                if ui
                    .button("📋 Copy to Clipboard")
                    .on_hover_text("Copy image directly to system clipboard")
                    .clicked()
                {
                    app.request_canvas_export(PathBuf::new(), true);
                    should_close = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_close {
        is_open = false;
    }
    app.show_export_modal = is_open;
}

pub fn generate_export_filename(var_name: &str, format: ExportFormat) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let safe_var = var_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    format!("octant_{}_{}.{}", safe_var, now, format.extension())
}

/// Shows the floating success toast notification with a "Reveal in Finder/Folder" action button.
pub fn show_export_toast(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if let Some(ref toast) = app.export_toast {
        let elapsed = toast.timestamp.elapsed().as_secs_f32();
        if elapsed > 6.0 {
            app.export_toast = None;
            return;
        }

        // Fade out smoothly during the last second
        let alpha = if elapsed > 5.0 {
            ((6.0 - elapsed) * 240.0).clamp(0.0, 240.0) as u8
        } else {
            240
        };

        let mut dismiss = false;
        let mut reveal = false;
        let file_path = toast.file_path.clone();
        let filename = toast.filename.clone();

        let toast_pos = egui::pos2(
            (canvas_rect.right() - 340.0).max(canvas_rect.left() + 10.0),
            (canvas_rect.bottom() - 56.0).max(canvas_rect.top() + 10.0),
        );

        egui::Area::new(egui::Id::new("octant_export_toast_area"))
            .fixed_pos(toast_pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_black_alpha(alpha))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 200, 255)))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("📸 Saved")
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new(&filename)
                                    .small()
                                    .color(egui::Color32::from_rgb(0, 220, 255)),
                            );

                            #[cfg(target_os = "macos")]
                            let reveal_label = "📂 Reveal in Finder";
                            #[cfg(not(target_os = "macos"))]
                            let reveal_label = "📂 Open Folder";

                            if ui
                                .button(egui::RichText::new(reveal_label).small().strong())
                                .on_hover_text(format!("Show in folder:\n{}", file_path.display()))
                                .clicked()
                            {
                                reveal = true;
                                dismiss = true;
                            }

                            if ui.small_button("✕").clicked() {
                                dismiss = true;
                            }
                        });
                    });
            });

        if reveal {
            crate::export::reveal_in_file_manager(&file_path);
        }
        if dismiss {
            app.export_toast = None;
        }

        ctx.request_repaint();
    }
}
