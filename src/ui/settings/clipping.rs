use super::gated;
use super::scale::show_scale_type_controls;
use super::support::OptionSupport;
use crate::app::OctantApp;
use crate::app::layers::{ColorStyle, LayerId};
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::color_picker::{ColorShape, ShapeColorPicker};
use crate::ui::flow_row;
use crate::ui::icons::{Icon, UiIconExt, icon_button_width};
use crate::ui::layer_label::LabelEditor;
use crate::ui::number_input::LocalizedInput;

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
    let categorical = support.categorical.is_yes();
    gated(ui, support.color_range, |ui| {
        show_color_range_controls(app, ui, id, categorical);
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
    if support.color_mapping.is_yes() {
        show_clip_color_pickers(ui, color, id);
    }
}

fn show_colorbar_label_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    ui.label(egui::RichText::new(tr(K::ColorbarLabel)).strong());
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
                .on_hover_text(tr(K::ResetLabelHover))
                .clicked()
        {
            edited = Some(None);
        }
    });
    if let (Some(custom), Some(layer)) = (edited, app.layers.get_mut(id)) {
        layer.color.custom_label = custom;
    }
}

/// The Color range title with the Categorical toggle (when the plot honors
/// it, `categorical`) and the lock and reset buttons, then the Min and Max
/// inputs.
fn show_color_range_controls(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    categorical: bool,
) {
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let color = &mut layer.color;
    let title = egui::RichText::new(tr(K::ColorRange)).strong();
    let title_w = flow_row::text_width(ui, title.clone(), egui::TextStyle::Body);
    let icon_w = icon_button_width(ui, "");
    let widths = [
        icon_w,
        icon_w,
        flow_row::button_width(ui, tr(K::Categorical)),
    ];
    let widths = if categorical {
        &widths[..]
    } else {
        &widths[..2]
    };
    let mut reset_range = false;
    flow_row::split(
        ui,
        title_w,
        widths,
        |ui| {
            ui.label(title);
        },
        |ui| {
            reset_range = ui
                .icon_button(Icon::Reset, "")
                .on_hover_text(tr(K::ResetBoundsHover))
                .clicked();
            let lock_icon = if color.lock_bounds {
                Icon::Lock
            } else {
                Icon::Unlock
            };
            if ui
                .icon_button(lock_icon, "")
                .on_hover_text(tr(K::LockBoundsHover))
                .clicked()
            {
                color.lock_bounds = !color.lock_bounds;
            }
            if categorical {
                ui.toggle_value(&mut color.categorical, tr(K::Categorical))
                    .on_hover_text(tr(K::CategoricalHover));
            }
        },
    );
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| range_inputs(ui, color));
    if (color.custom_label.is_some() || color.lock_bounds)
        && ui
            .icon_button(Icon::Reset, tr(K::ResetAllColorbar))
            .on_hover_text(tr(K::ResetAllColorbarHover))
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
        (K::RangeMin, &mut color.range_min),
        (K::RangeMax, &mut color.range_max),
    ] {
        ui.label(tr(label));
        let input = egui::DragValue::new(value)
            .localized()
            .speed(speed)
            .custom_formatter(|val, _| crate::ui::colorbar::format_scientific_tick(val as f32));
        if ui.add(input).changed() {
            color.lock_bounds = true;
        }
    }
}

fn show_nan_color_picker(ui: &mut egui::Ui, color: &mut ColorStyle, id: LayerId) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut color.use_nan_color, tr(K::NanColor))
            .on_hover_text(tr(K::NanColorHover));
        if color.use_nan_color {
            let picker = ShapeColorPicker::new(
                ("settings_nan_color_picker", id),
                &mut color.nan_color,
                ColorShape::Rect(3.0),
            );
            show_picker(ui, picker, tr(K::NanColorPickerHover));
        }
    });
}

fn show_clip_color_pickers(ui: &mut egui::Ui, color: &mut ColorStyle, id: LayerId) {
    let clips = [
        (
            "low_clip",
            K::LowClip,
            K::LowClipHover,
            &mut color.use_lowclip,
            &mut color.lowclip_color,
            ColorShape::LeftTriangle,
            K::LowClipPickerHover,
        ),
        (
            "high_clip",
            K::HighClip,
            K::HighClipHover,
            &mut color.use_highclip,
            &mut color.highclip_color,
            ColorShape::RightTriangle,
            K::HighClipPickerHover,
        ),
    ];
    for (salt, label, hover, enabled, rgba, shape, tooltip) in clips {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.checkbox(enabled, tr(label)).on_hover_text(tr(hover));
            if *enabled {
                let picker = ShapeColorPicker::new(("settings_clip_picker", salt, id), rgba, shape);
                show_picker(ui, picker, tr(tooltip));
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
