use super::gated;
use super::scale::show_scale_type_controls;
use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::app::layers::{ColorStyle, LayerId};
use crate::ui::color_picker::{ColorShape, ShapeColorPicker};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::layer_label::LabelEditor;

/// Layer `id`'s colorbar label, color range, scale and the NaN and clip
/// colors, as far as its plot honors them.
pub(crate) fn show_color_settings(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    support: &OptionSupport,
) {
    show_colorbar_label_controls(app, ui, id);
    ui.add_space(4.0);
    // An overridden range overrides the whole mapping: one note says why.
    let mapped = support.color_mapping.is_yes();
    gated(ui, support.color_range, |ui| {
        show_color_range_controls(app, ui, id, mapped);
        ui.add_space(4.0);
        gated(ui, support.color_mapping, |ui| {
            if let Some(layer) = app.layers.get_mut(id) {
                show_scale_type_controls(ui, &mut layer.color, id);
            }
        });
    });
    ui.add_space(4.0);
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let color = &mut layer.color;
    gated(ui, support.nan_color, |ui| {
        show_nan_color_picker(ui, color, id)
    });
    if mapped {
        show_clip_color_pickers(ui, color, id);
    }
}

fn show_colorbar_label_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    ui.label(egui::RichText::new("Colorbar Label").strong());
    let Some(layer) = app.layers.get(id) else {
        return;
    };
    let editor = LabelEditor {
        id: egui::Id::new(("settings_colorbar_label", id)),
        width: 170.0,
        framed: true,
    };
    let mut edited = None;
    ui.horizontal(|ui| {
        edited = editor.show(ui, layer);
        if layer.color.custom_label.is_some()
            && ui
                .icon_button(Icon::Reset, "")
                .on_hover_text("Reset colorbar label to default")
                .clicked()
        {
            edited = Some(None);
        }
    });
    if let (Some(custom), Some(layer)) = (edited, app.layers.get_mut(id)) {
        layer.color.custom_label = custom;
    }
}

/// Min and max inputs with lock and reset; the Categorical toggle when the
/// colormap is `mapped`.
fn show_color_range_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId, mapped: bool) {
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let color = &mut layer.color;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Color Range").strong());
        if !mapped {
            return;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.toggle_value(&mut color.categorical, "Categorical")
                .on_hover_text("Discrete colorbar (auto-detects unique values or 10 equal bins).");
        });
    });
    ui.add_space(2.0);
    let mut reset_range = false;
    ui.horizontal(|ui| {
        range_inputs(ui, color);
        let lock_icon = if color.lock_bounds {
            Icon::Lock
        } else {
            Icon::Unlock
        };
        if ui
            .icon_button(lock_icon, "")
            .on_hover_text("Lock min/max so color mapping stays fixed across timesteps.")
            .clicked()
        {
            color.lock_bounds = !color.lock_bounds;
        }
        reset_range = ui
            .icon_button(Icon::Reset, "")
            .on_hover_text("Reset bounds to current slice/dataset min and max defaults")
            .clicked();
    });
    if (color.custom_label.is_some() || color.lock_bounds)
        && ui
            .icon_button(Icon::Reset, "Reset All Colorbar Defaults")
            .on_hover_text("Reset both colorbar label and range to default values")
            .clicked()
    {
        color.custom_label = None;
        reset_range = true;
    }
    if reset_range {
        app.reset_layer_color_range(id);
    }
}

/// The Min and Max inputs; editing either locks the range.
fn range_inputs(ui: &mut egui::Ui, color: &mut ColorStyle) {
    let speed = ((color.range_max - color.range_min).abs() / 100.0).max(1e-4);
    for (label, value) in [
        ("Min:", &mut color.range_min),
        ("Max:", &mut color.range_max),
    ] {
        ui.label(label);
        let input = egui::DragValue::new(value)
            .speed(speed)
            .custom_formatter(|val, _| crate::ui::colorbar::format_scientific_tick(val as f32))
            .custom_parser(|s| s.trim().parse::<f64>().ok());
        if ui.add(input).changed() {
            color.lock_bounds = true;
        }
    }
}

fn show_nan_color_picker(ui: &mut egui::Ui, color: &mut ColorStyle, id: LayerId) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut color.use_nan_color, "NaN Color")
            .on_hover_text("If unchecked, NaN/Inf values render transparently.");
        if color.use_nan_color {
            let picker = ShapeColorPicker::new(
                ("settings_nan_color_picker", id),
                &mut color.nan_color,
                ColorShape::Rect(3.0),
            );
            show_picker(ui, picker, "NaN color. Click to select color.");
        }
    });
}

fn show_clip_color_pickers(ui: &mut egui::Ui, color: &mut ColorStyle, id: LayerId) {
    let clips = [
        (
            "Low Clip",
            "Values < cmin clipped to this color.",
            &mut color.use_lowclip,
            &mut color.lowclip_color,
            ColorShape::LeftTriangle,
            "Low Clip color (< Min). Click to select color.",
        ),
        (
            "High Clip",
            "Values > cmax clipped to this color.",
            &mut color.use_highclip,
            &mut color.highclip_color,
            ColorShape::RightTriangle,
            "High Clip color (> Max). Click to select color.",
        ),
    ];
    for (label, hover, enabled, rgba, shape, tooltip) in clips {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.checkbox(enabled, label).on_hover_text(hover);
            if *enabled {
                let picker =
                    ShapeColorPicker::new(("settings_clip_picker", label, id), rgba, shape);
                show_picker(ui, picker, tooltip);
            }
        });
    }
}

/// A small color swatch picker at the row's right end.
fn show_picker(ui: &mut egui::Ui, picker: ShapeColorPicker<'_>, tooltip: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        picker
            .size(egui::vec2(18.0, 16.0))
            .tooltip(tooltip)
            .anchor_offset(egui::vec2(-240.0, -100.0))
            .show(ui);
    });
}
