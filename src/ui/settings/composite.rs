//! RGB, CMYK, and Multi-Channel overlay controls for 2D settings panel.

use crate::app::OctantApp;

/// Render composite controls (Multi-Channel bioimaging overlay, CMYK, or standard 3-band RGB).
pub(crate) fn show_composite_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.separator();
    let is_cmyk = app.is_cmyk();
    let has_mc = !app.composite_channel_configs.is_empty();

    let label = if has_mc {
        "Multi-Channel Overlay"
    } else if is_cmyk {
        "CMYK Composite"
    } else {
        "RGB Composite"
    };

    let tooltip = if has_mc {
        "Overlays multiple channels additively, each rendered with its unique color tint."
    } else if is_cmyk {
        "Composites 4-channel Cyan, Magenta, Yellow, Black (CMYK) into Truecolor RGB."
    } else {
        "Composites selected 3 channels into Truecolor RGB."
    };

    let mut rgb_mode = app.rgb_composite_mode;
    if ui
        .checkbox(&mut rgb_mode, label)
        .on_hover_text(tooltip)
        .changed()
    {
        app.rgb_composite_mode = rgb_mode;
        app.active_colormap = if rgb_mode { 1000 } else { 0 };
        app.load_selected_variable_block();
    }

    if app.rgb_composite_mode {
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

fn show_multichannel_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Channels:").small().strong());
        if ui.small_button("All").clicked() {
            for cfg in &mut app.composite_channel_configs {
                cfg.visible = true;
            }
            changed = true;
        }
        if ui.small_button("None").clicked() {
            for cfg in &mut app.composite_channel_configs {
                cfg.visible = false;
            }
            changed = true;
        }
    });

    egui::Grid::new("multichannel_overlay_grid")
        .num_columns(3)
        .spacing([6.0, 3.0])
        .show(ui, |ui| {
            for cfg in &mut app.composite_channel_configs {
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

    if changed {
        app.load_selected_variable_block();
    }
}

fn show_standard_rgb_controls(app: &mut OctantApp, ui: &mut egui::Ui) {
    let num_b = app.num_bands();
    let channel_labels: Vec<String> = app
        .selected_variable_info()
        .or_else(|| app.plotted_variable_info())
        .and_then(|v| v.attributes.get("omero_channels"))
        .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();

    let get_channel_title = |b: usize| {
        if let Some(name) = channel_labels.get(b) {
            format!("{}: {name}", b + 1)
        } else {
            format!("Band {}", b + 1)
        }
    };

    let mut changed = false;
    ui.horizontal(|ui| {
        let channels = [
            (0, "R:", egui::Color32::from_rgb(255, 100, 100), "rgb_r_ch"),
            (1, "G:", egui::Color32::from_rgb(100, 255, 100), "rgb_g_ch"),
            (2, "B:", egui::Color32::from_rgb(100, 150, 255), "rgb_b_ch"),
        ];
        for (idx, label, color, salt) in channels {
            ui.label(egui::RichText::new(label).color(color));
            let mut ch = app.rgb_composite_channels[idx].min(num_b.saturating_sub(1));
            egui::ComboBox::from_id_salt(salt)
                .selected_text(get_channel_title(ch))
                .show_ui(ui, |ui| {
                    for b in 0..num_b {
                        if ui.selectable_label(ch == b, get_channel_title(b)).clicked() {
                            ch = b;
                        }
                    }
                });
            if ch != app.rgb_composite_channels[idx] {
                app.rgb_composite_channels[idx] = ch;
                changed = true;
            }
        }
    });
    if changed {
        app.load_selected_variable_block();
    }
}
