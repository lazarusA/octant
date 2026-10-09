//! The R, G and B band selects of a layer's standard RGB composite.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::utils::stack_str;

/// Width reserved for each "R:" / "G:" / "B:" channel label.
const RGB_LABEL_W: f32 = 16.0;
/// Narrowest a channel select may be before the row stacks vertically.
const RGB_MIN_COMBO_W: f32 = 84.0;
/// R, G and B: label, label color and widget id salt.
const RGB_CHANNELS: [(&str, egui::Color32, &str); 3] = [
    ("R:", egui::Color32::from_rgb(255, 100, 100), "rgb_r_ch"),
    ("G:", egui::Color32::from_rgb(100, 255, 100), "rgb_g_ch"),
    ("B:", egui::Color32::from_rgb(100, 150, 255), "rgb_b_ch"),
];

/// One select per RGB channel: on a single row when all three fit at a usable
/// width, otherwise stacked, one channel per row.
pub(super) fn show_standard_rgb_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let num_b = app.layer_num_bands(id);
    let (c_start, c_end) = super::composite::get_selected_channel_range(app, id);
    let min_b = c_start.min(num_b.saturating_sub(1));
    let max_b = c_end.min(num_b.saturating_sub(1)).max(min_b);

    let Some(layer) = app.layers.get(id) else {
        return;
    };
    let mut selected = layer.composite.rgb_channels.map(|b| b.clamp(min_b, max_b));
    let names = app
        .layer_variable_info(id)
        .and_then(|v| v.attributes.get("omero_channels"))
        .map(String::as_str);

    let spacing = ui.spacing().item_spacing.x;
    let per_channel = RGB_LABEL_W + spacing + RGB_MIN_COMBO_W + spacing;
    let inline = ui.available_width() >= per_channel * RGB_CHANNELS.len() as f32;
    let mut select = |ui: &mut egui::Ui, idx: usize| {
        let salt = (RGB_CHANNELS[idx].2, id);
        rgb_channel_select(
            ui,
            idx,
            salt,
            &mut selected[idx],
            names,
            min_b..=max_b,
            inline,
        );
    };
    if inline {
        ui.horizontal(|ui| (0..RGB_CHANNELS.len()).for_each(|idx| select(ui, idx)));
    } else {
        for idx in 0..RGB_CHANNELS.len() {
            ui.horizontal(|ui| select(ui, idx));
        }
    }

    let composite = &mut app.layers.get_or_base_mut(id).composite;
    if selected != composite.rgb_channels {
        composite.rgb_channels = selected;
        app.load_layer_block(id);
    }
}

/// Label plus band select for RGB channel `idx`. Inline rows split the
/// remaining width evenly between the channels still to draw; stacked rows
/// give the select everything after the label.
fn rgb_channel_select(
    ui: &mut egui::Ui,
    idx: usize,
    salt: (&str, LayerId),
    band: &mut usize,
    names: Option<&str>,
    bands: std::ops::RangeInclusive<usize>,
    inline: bool,
) {
    let (label, color, _) = RGB_CHANNELS[idx];
    let spacing = ui.spacing().item_spacing.x;
    let share = if inline {
        (ui.available_width() + spacing) / (RGB_CHANNELS.len() - idx) as f32 - spacing
    } else {
        ui.available_width()
    };
    ui.add_sized(
        [RGB_LABEL_W, ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(label).color(color)),
    );

    let mut buf = [0u8; 64];
    egui::ComboBox::from_id_salt(salt)
        .width((share - RGB_LABEL_W - spacing).max(RGB_MIN_COMBO_W))
        .selected_text(band_title(&mut buf, names, *band))
        .show_ui(ui, |ui| {
            for b in bands {
                let mut buf = [0u8; 64];
                if ui
                    .selectable_label(*band == b, band_title(&mut buf, names, b))
                    .clicked()
                {
                    *band = b;
                }
            }
        });
}

/// "N: name" from the comma-separated `names` attribute, else "Band N",
/// written into `buf` without allocating.
fn band_title<'a>(buf: &'a mut [u8; 64], names: Option<&str>, band: usize) -> &'a str {
    let number = band + 1;
    match names.and_then(|n| n.split(',').nth(band)).map(str::trim) {
        Some(name) => stack_str(buf, format_args!("{number}: {name}")),
        None => stack_str(buf, format_args!("Band {number}")),
    }
}
