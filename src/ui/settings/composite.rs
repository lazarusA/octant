//! RGB, CMYK, and Multi-Channel overlay controls for 2D settings panel.

use crate::app::OctantApp;
use crate::ui::hover::composite::composite_labels;
use crate::utils::stack_str;

/// Render composite controls (Multi-Channel bioimaging overlay, CMYK, or standard 3-band RGB).
pub(crate) fn show_composite_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.separator();
    let is_cmyk = app.is_cmyk();
    let is_tiff = app.is_geotiff();
    let has_mc = !app.layers.base.composite.channel_configs.is_empty() && !is_tiff;

    let (label, tooltip) = toggle_text(has_mc, is_cmyk);

    // Name the band combination once it is drawn, as the hover card does.
    let mut label_buf = [0u8; 48];
    let label = if app.layers.base.composite.enabled && !has_mc && !is_cmyk {
        let meta = app.plotted().metadata.as_ref();
        let kind = composite_labels(app, ui.ctx(), meta, app.plotted_variable_info()).kind;
        stack_str(&mut label_buf, format_args!("{label} ({})", kind.label()))
    } else {
        label
    };

    let mut rgb_mode = app.layers.base.composite.enabled;
    if ui
        .checkbox(&mut rgb_mode, label)
        .on_hover_text(tooltip)
        .changed()
    {
        app.layers.base.composite.enabled = rgb_mode;
        app.load_selected_variable_block();
    }

    if app.layers.base.composite.enabled {
        if has_mc {
            show_multichannel_controls(app, ui);
        } else if is_cmyk {
            ui.label(
                egui::RichText::new("Auto-mapped channels: C (1), M (2), Y (3), K (4)")
                    .small()
                    .weak(),
            );
        } else {
            show_standard_rgb_controls(app, ui);
        }
    }
}

/// The composite checkbox's label and tooltip for the mode the dataset supports.
fn toggle_text(has_mc: bool, is_cmyk: bool) -> (&'static str, &'static str) {
    if has_mc {
        (
            "Multi-Channel Overlay",
            "Overlays multiple channels additively, each rendered with its unique color tint.",
        )
    } else if is_cmyk {
        (
            "CMYK Composite",
            "Composites 4-channel Cyan, Magenta, Yellow, Black (CMYK) into Truecolor RGB.",
        )
    } else {
        (
            "RGB Composite",
            "Composites selected 3 channels into Truecolor RGB.",
        )
    }
}

fn get_selected_channel_range(app: &OctantApp) -> (usize, usize) {
    if let Some(c_idx) = app.channel_dim_index() {
        app.get_effective_dim_range(c_idx)
    } else {
        (0, usize::MAX)
    }
}

fn show_multichannel_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Channels:").small().strong());
        if ui.small_button("All").clicked() {
            for cfg in &mut app.layers.base.composite.channel_configs {
                cfg.visible = true;
            }
            changed = true;
        }
        if ui.small_button("None").clicked() {
            for cfg in &mut app.layers.base.composite.channel_configs {
                cfg.visible = false;
            }
            changed = true;
        }
    });

    egui::ScrollArea::vertical()
        .max_height(100.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            egui::Grid::new("multichannel_overlay_grid")
                .num_columns(3)
                .spacing([6.0, 3.0])
                .show(ui, |ui| {
                    for cfg in &mut app.layers.base.composite.channel_configs {
                        if ui.checkbox(&mut cfg.visible, "").changed() {
                            changed = true;
                        }
                        let mut color_f32 = [
                            cfg.color_rgb[0] as f32 / 255.0,
                            cfg.color_rgb[1] as f32 / 255.0,
                            cfg.color_rgb[2] as f32 / 255.0,
                            1.0,
                        ];
                        let initial_f32 = color_f32;
                        crate::ui::color_picker::ShapeColorPicker::new(
                            ("mc_color_picker", cfg.index),
                            &mut color_f32,
                            crate::ui::color_picker::ColorShape::Circle,
                        )
                        .size(egui::vec2(14.0, 14.0))
                        .tooltip("Click to customize channel tint color")
                        .show(ui);

                        if color_f32 != initial_f32 {
                            cfg.color_rgb = [
                                (color_f32[0] * 255.0).round().clamp(0.0, 255.0) as u8,
                                (color_f32[1] * 255.0).round().clamp(0.0, 255.0) as u8,
                                (color_f32[2] * 255.0).round().clamp(0.0, 255.0) as u8,
                            ];
                            changed = true;
                        }

                        let label_text = format!("{}: {}", cfg.index + 1, cfg.name);
                        ui.label(egui::RichText::new(label_text).small());
                        ui.end_row();
                    }
                });
        });

    if changed {
        app.load_selected_variable_block();
    }
}

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
fn show_standard_rgb_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let num_b = app.num_bands();
    let (c_start, c_end) = get_selected_channel_range(app);
    let min_b = c_start.min(num_b.saturating_sub(1));
    let max_b = c_end.min(num_b.saturating_sub(1)).max(min_b);

    let mut selected = app
        .layers
        .base
        .composite
        .rgb_channels
        .map(|b| b.clamp(min_b, max_b));
    let names = app
        .plotted_variable_info()
        .and_then(|v| v.attributes.get("omero_channels"))
        .map(String::as_str);

    let spacing = ui.spacing().item_spacing.x;
    let per_channel = RGB_LABEL_W + spacing + RGB_MIN_COMBO_W + spacing;
    let inline = ui.available_width() >= per_channel * RGB_CHANNELS.len() as f32;
    let mut select = |ui: &mut egui::Ui, idx: usize| {
        rgb_channel_select(ui, idx, &mut selected[idx], names, min_b..=max_b, inline);
    };
    if inline {
        ui.horizontal(|ui| (0..RGB_CHANNELS.len()).for_each(|idx| select(ui, idx)));
    } else {
        for idx in 0..RGB_CHANNELS.len() {
            ui.horizontal(|ui| select(ui, idx));
        }
    }

    if selected != app.layers.base.composite.rgb_channels {
        app.layers.base.composite.rgb_channels = selected;
        app.load_selected_variable_block();
    }
}

/// Label plus band select for RGB channel `idx`. Inline rows split the
/// remaining width evenly between the channels still to draw; stacked rows
/// give the select everything after the label.
fn rgb_channel_select(
    ui: &mut egui::Ui,
    idx: usize,
    band: &mut usize,
    names: Option<&str>,
    bands: std::ops::RangeInclusive<usize>,
    inline: bool,
) {
    let (label, color, salt) = RGB_CHANNELS[idx];
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
