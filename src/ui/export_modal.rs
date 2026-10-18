use crate::app::OctantApp;
use crate::export::{ExportFormat, ExportTarget};
use crate::i18n::{CommonKey, ExportKey, tr};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::keyboard::windows;
use crate::ui::number_input::LocalizedInput;
use std::path::PathBuf;

/// Shows the floating modal dialog for saving and exporting the canvas/figure.
pub fn show_export_modal(app: &mut OctantApp, ctx: &egui::Context) {
    let window_id = egui::Id::unique("export_modal_window");
    let opened = windows::follow_window(ctx, window_id, app.show_export_modal);
    if !app.show_export_modal {
        return;
    }

    let mut should_close = false;

    let window = egui::Window::new(tr(ExportKey::DialogTitle))
        .id(window_id)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_width(380.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            windows::trap(ui, opened);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr(ExportKey::DialogTitle)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.close_button(tr(CommonKey::CloseEsc)).clicked() {
                        should_close = true;
                    }
                });
            });
            ui.separator();
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

            // 1. Format Selection Tabs
            ui.label(egui::RichText::new(tr(ExportKey::FormatHeading)).strong());
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
                    ui.label(tr(ExportKey::JpegQuality));
                    ui.add(
                        egui::Slider::new(&mut app.export_settings.jpeg_quality, 10..=100)
                            .localized()
                            .suffix("%"),
                    );
                });
            }

            ui.separator();

            // 2. Export Target (Full Canvas vs ROI)
            ui.label(egui::RichText::new(tr(ExportKey::AreaHeading)).strong());
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut app.export_settings.target,
                    ExportTarget::FullCanvas,
                    tr(ExportKey::FullCanvas),
                );
                ui.radio_value(
                    &mut app.export_settings.target,
                    ExportTarget::RoiCrop,
                    tr(ExportKey::FramedRoi),
                );
            });

            if app.export_settings.target == ExportTarget::RoiCrop {
                ui.horizontal(|ui| {
                    if ui
                        .button(if app.show_crop_overlay {
                            tr(ExportKey::HideGuides)
                        } else {
                            tr(ExportKey::ShowGuides)
                        })
                        .clicked()
                    {
                        app.show_crop_overlay = !app.show_crop_overlay;
                    }
                });
            }

            ui.separator();

            // 3. Export Directory & Filename Preview
            ui.label(egui::RichText::new(tr(ExportKey::DestinationHeading)).strong());
            ui.horizontal(|ui| {
                ui.label(tr(ExportKey::Folder));
                ui.text_edit_singleline(&mut app.export_settings.export_dir);
            });

            let var_name = app
                .plotted_variable_info()
                .map(|v| v.name.as_str())
                .unwrap_or("plot");
            let default_name =
                crate::export::generate_export_filename(var_name, app.export_settings.format);

            let mut preview = [0; 1024];
            let preview = crate::i18n::fill(
                &mut preview,
                tr(ExportKey::Preview),
                &[&app.export_settings.export_dir, &default_name],
            );
            ui.label(egui::RichText::new(preview).small().weak());

            ui.separator();

            // 4. Action Buttons
            ui.horizontal(|ui| {
                if ui
                    .icon_button(Icon::Save, tr(ExportKey::SaveFigure))
                    .on_hover_text(tr(ExportKey::SaveFigureHover))
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
                    .icon_button(Icon::Clipboard, tr(ExportKey::CopyToClipboard))
                    .on_hover_text(tr(ExportKey::CopyToClipboardHover))
                    .clicked()
                {
                    app.request_canvas_export(PathBuf::new(), true);
                    should_close = true;
                }

                if ui.button(tr(CommonKey::Cancel)).clicked() {
                    should_close = true;
                }
            });
        });

    let dismissed = window.is_some_and(|w| windows::dismissed(ctx, w.response.rect));
    if should_close || dismissed {
        app.show_export_modal = false;
    }
}
