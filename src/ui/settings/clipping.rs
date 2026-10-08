use super::gated;
use super::scale::show_scale_type_controls;
use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

/// Colorbar label, color range, scale and the NaN and clip colors, as far as
/// the plot honors them.
pub(crate) fn show_color_settings(app: &mut OctantApp, ui: &mut egui::Ui, support: &OptionSupport) {
    show_colorbar_label_controls(app, ui);
    ui.add_space(4.0);
    // An overridden range overrides the whole mapping: one note says why.
    let mapped = support.color_mapping.is_yes();
    gated(ui, support.color_range, |ui| {
        show_color_range_controls(app, ui, mapped);
        ui.add_space(4.0);
        gated(ui, support.color_mapping, |ui| {
            show_scale_type_controls(app, ui)
        });
    });
    ui.add_space(4.0);
    gated(ui, support.nan_color, |ui| show_nan_color_picker(app, ui));
    if mapped {
        show_clip_color_pickers(app, ui);
    }
}

fn show_colorbar_label_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Colorbar Label").strong());
    let default_label = app.default_colorbar_label();
    let mut label_buf = app.colorbar_label();
    let has_custom_label = app.layers.base.color.custom_label.is_some();

    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut label_buf)
                .hint_text(&default_label)
                .desired_width(170.0),
        );
        if resp.changed() {
            if label_buf.trim().is_empty() || label_buf == default_label {
                app.layers.base.color.custom_label = None;
            } else {
                app.layers.base.color.custom_label = Some(label_buf);
            }
        }

        if has_custom_label
            && ui
                .icon_button(Icon::Reset, "")
                .on_hover_text("Reset colorbar label to default")
                .clicked()
        {
            app.reset_colorbar_label();
        }
    });
}

/// Min and max inputs with lock and reset; the Categorical toggle when the
/// colormap is `mapped`.
fn show_color_range_controls(app: &mut OctantApp, ui: &mut egui::Ui, mapped: bool) {
    let range_speed = ((app.layers.base.color.range_max - app.layers.base.color.range_min).abs()
        / 100.0)
        .max(1e-4);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Color Range").strong());
        if !mapped {
            return;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.toggle_value(&mut app.layers.base.color.categorical, "Categorical")
                .on_hover_text("Discrete colorbar (auto-detects unique values or 10 equal bins).");
        });
    });

    ui.add_space(2.0);

    ui.horizontal(|ui| {
        ui.label("Min:");
        if ui
            .add(
                egui::DragValue::new(&mut app.layers.base.color.range_min)
                    .speed(range_speed)
                    .custom_formatter(|val, _| {
                        crate::ui::colorbar::format_scientific_tick(val as f32)
                    })
                    .custom_parser(|s| s.trim().parse::<f64>().ok()),
            )
            .changed()
        {
            app.layers.base.color.lock_bounds = true;
        }

        ui.label("Max:");
        if ui
            .add(
                egui::DragValue::new(&mut app.layers.base.color.range_max)
                    .speed(range_speed)
                    .custom_formatter(|val, _| {
                        crate::ui::colorbar::format_scientific_tick(val as f32)
                    })
                    .custom_parser(|s| s.trim().parse::<f64>().ok()),
            )
            .changed()
        {
            app.layers.base.color.lock_bounds = true;
        }

        let lock_icon = if app.layers.base.color.lock_bounds {
            Icon::Lock
        } else {
            Icon::Unlock
        };
        if ui
            .icon_button(lock_icon, "")
            .on_hover_text("Lock min/max so color mapping stays fixed across timesteps.")
            .clicked()
        {
            app.layers.base.color.lock_bounds = !app.layers.base.color.lock_bounds;
        }

        if ui
            .icon_button(Icon::Reset, "")
            .on_hover_text("Reset bounds to current slice/dataset min and max defaults")
            .clicked()
        {
            app.reset_color_range();
        }
    });

    if (app.layers.base.color.custom_label.is_some() || app.layers.base.color.lock_bounds)
        && ui
            .icon_button(Icon::Reset, "Reset All Colorbar Defaults")
            .on_hover_text("Reset both colorbar label and range to default values")
            .clicked()
    {
        app.reset_colorbar_label();
        app.reset_color_range();
    }
}

fn show_nan_color_picker(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.layers.base.color.use_nan_color, "NaN Color")
            .on_hover_text("If unchecked, NaN/Inf values render transparently.");
        if app.layers.base.color.use_nan_color {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                crate::ui::color_picker::ShapeColorPicker::new(
                    "settings_nan_color_picker",
                    &mut app.layers.base.color.nan_color,
                    crate::ui::color_picker::ColorShape::Rect(3.0),
                )
                .size(egui::vec2(18.0, 16.0))
                .tooltip("NaN color. Click to select color.")
                .anchor_offset(egui::vec2(-240.0, -100.0))
                .show(ui);
            });
        }
    });
}

fn show_clip_color_pickers(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.layers.base.color.use_lowclip, "Low Clip")
            .on_hover_text("Values < cmin clipped to this color.");
        if app.layers.base.color.use_lowclip {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                crate::ui::color_picker::ShapeColorPicker::new(
                    "settings_lowclip_color_picker",
                    &mut app.layers.base.color.lowclip_color,
                    crate::ui::color_picker::ColorShape::LeftTriangle,
                )
                .size(egui::vec2(18.0, 16.0))
                .tooltip("Low Clip color (< Min). Click to select color.")
                .anchor_offset(egui::vec2(-240.0, -100.0))
                .show(ui);
            });
        }
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.layers.base.color.use_highclip, "High Clip")
            .on_hover_text("Values > cmax clipped to this color.");
        if app.layers.base.color.use_highclip {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                crate::ui::color_picker::ShapeColorPicker::new(
                    "settings_highclip_color_picker",
                    &mut app.layers.base.color.highclip_color,
                    crate::ui::color_picker::ColorShape::RightTriangle,
                )
                .size(egui::vec2(18.0, 16.0))
                .tooltip("High Clip color (> Max). Click to select color.")
                .anchor_offset(egui::vec2(-240.0, -100.0))
                .show(ui);
            });
        }
    });
}
