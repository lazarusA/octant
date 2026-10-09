//! RGB, CMYK, and Multi-Channel overlay controls for 2D settings panel.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::ui::hover::composite::composite_labels;
use crate::utils::stack_str;

/// Layer `id`'s composite controls (Multi-Channel bioimaging overlay, CMYK,
/// or standard 3-band RGB), while its variable has bands to compose.
pub(crate) fn show_composite_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let Some(layer) = app.layers.get(id) else {
        return;
    };
    if !app.layer_has_rgb_bands(id) && layer.composite.channel_configs.is_empty() {
        app.layers.get_or_base_mut(id).composite.enabled = false;
        return;
    }
    let is_cmyk = app.layer_is_cmyk(id);
    let is_tiff = app.layer_is_geotiff(id);
    let has_mc = !layer.composite.channel_configs.is_empty() && !is_tiff;

    let (label, tooltip) = toggle_text(has_mc, is_cmyk);

    // Name the band combination once it is drawn, as the hover card does.
    let mut label_buf = [0u8; 48];
    // The band names cache follows the base layer (the hover card's).
    let enabled = layer.composite.enabled;
    let label = if enabled && !has_mc && !is_cmyk && id == LayerId::BASE {
        let meta = app.plotted().metadata.as_ref();
        let kind = composite_labels(app, ui.ctx(), meta, app.plotted_variable_info()).kind;
        stack_str(&mut label_buf, format_args!("{label} ({})", kind.label()))
    } else {
        label
    };

    let mut rgb_mode = enabled;
    if ui
        .checkbox(&mut rgb_mode, label)
        .on_hover_text(tooltip)
        .changed()
    {
        app.layers.get_or_base_mut(id).composite.enabled = rgb_mode;
        app.load_layer_block(id);
    }

    if rgb_mode {
        if has_mc {
            show_multichannel_controls(app, ui, id);
        } else if is_cmyk {
            ui.label(
                egui::RichText::new("Auto-mapped channels: C (1), M (2), Y (3), K (4)")
                    .small()
                    .weak(),
            );
        } else {
            super::composite_rgb::show_standard_rgb_controls(app, ui, id);
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

/// Layer `id`'s selected band range, or every band without a band dimension.
pub(super) fn get_selected_channel_range(app: &OctantApp, id: LayerId) -> (usize, usize) {
    match app.layer_channel_dim(id) {
        Some(c_idx) => app.layer_dim_range(id, c_idx),
        None => (0, usize::MAX),
    }
}

fn show_multichannel_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let composite = &mut app.layers.get_or_base_mut(id).composite;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Channels:").small().strong());
        if ui.small_button("All").clicked() {
            for cfg in &mut composite.channel_configs {
                cfg.visible = true;
            }
            changed = true;
        }
        if ui.small_button("None").clicked() {
            for cfg in &mut composite.channel_configs {
                cfg.visible = false;
            }
            changed = true;
        }
    });

    egui::ScrollArea::vertical()
        .max_height(100.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            egui::Grid::new(("multichannel_overlay_grid", id))
                .num_columns(3)
                .spacing([6.0, 3.0])
                .show(ui, |ui| {
                    for cfg in &mut composite.channel_configs {
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
                            ("mc_color_picker", id, cfg.index),
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
        app.load_layer_block(id);
    }
}
