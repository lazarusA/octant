use crate::app::OctantApp;
use crate::export::{ExportFormat, ExportTarget};
use crate::ui::icons::{Icon, UiIconExt};
use std::path::PathBuf;

/// Shows the floating modal dialog for saving and exporting the canvas/figure.
pub fn show_export_modal(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_export_modal {
        return;
    }

    let mut should_close = false;

    egui::Window::new("Save & Export Figure")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_width(380.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Save & Export Figure").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.close_button("Close (Esc)").clicked() {
                        should_close = true;
                    }
                });
            });
            ui.separator();
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
                    "Full Canvas (with Overlays)",
                );
                ui.radio_value(
                    &mut app.export_settings.target,
                    ExportTarget::RoiCrop,
                    "Framed ROI",
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
            let default_name =
                crate::export::generate_export_filename(var_name, app.export_settings.format);

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
                    .icon_button(Icon::Save, "Save Figure")
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
                    .icon_button(Icon::Clipboard, "Copy to Clipboard")
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

    if should_close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.show_export_modal = false;
    }
}
