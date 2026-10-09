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
        if let Some(layer) = app.layers.get_mut(id) {
            layer.composite.enabled = false;
        }
        return;
    }
    let is_cmyk = app.layer_is_cmyk(id);
    let is_tiff = app.layer_is_geotiff(id);
    let has_mc = !layer.composite.channel_configs.is_empty() && !is_tiff;

    let rgb_mode = composite_toggle(app, ui, id, has_mc, is_cmyk);
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

/// Layer `id`'s composite checkbox, named for its mode (and, on the base
/// layer, its band combination); reloads the layer when toggled. Whether the
/// composite is on.
fn composite_toggle(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    id: LayerId,
    has_mc: bool,
    is_cmyk: bool,
) -> bool {
    let enabled = app.layers.get(id).is_some_and(|l| l.composite.enabled);
    let (label, tooltip) = toggle_text(has_mc, is_cmyk);
    // The band names cache follows the base layer (the hover card's).
    let mut label_buf = [0u8; 48];
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
        if let Some(layer) = app.layers.get_mut(id) {
            layer.composite.enabled = rgb_mode;
        }
        app.load_layer_block(id);
    }
    rgb_mode
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
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let configs = &mut layer.composite.channel_configs;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Channels:").small().strong());
        for (label, visible) in [("All", true), ("None", false)] {
            if ui.small_button(label).clicked() {
                configs.iter_mut().for_each(|cfg| cfg.visible = visible);
                changed = true;
            }
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
                    for cfg in configs.iter_mut() {
                        changed |= channel_row(ui, cfg, id);
                        ui.end_row();
                    }
                });
        });
    if changed {
        app.load_layer_block(id);
    }
}

/// One channel of layer `id`'s multi-channel overlay: visibility, tint and
/// "N: name". Whether the channel changed.
fn channel_row(
    ui: &mut egui::Ui,
    cfg: &mut crate::data::slicing::ChannelColorConfig,
    id: LayerId,
) -> bool {
    let mut changed = ui.checkbox(&mut cfg.visible, "").changed();
    let [r, g, b] = cfg.color_rgb.map(|c| c as f32 / 255.0);
    let mut color = [r, g, b, 1.0];
    let initial = color;
    crate::ui::color_picker::ShapeColorPicker::new(
        ("mc_color_picker", id, cfg.index),
        &mut color,
        crate::ui::color_picker::ColorShape::Circle,
    )
    .size(egui::vec2(14.0, 14.0))
    .tooltip("Click to customize channel tint color")
    .show(ui);
    if color != initial {
        let to_u8 = |c: f32| (c * 255.0).round().clamp(0.0, 255.0) as u8;
        cfg.color_rgb = [to_u8(color[0]), to_u8(color[1]), to_u8(color[2])];
        changed = true;
    }
    let mut buf = [0u8; 96];
    let label = stack_str(&mut buf, format_args!("{}: {}", cfg.index + 1, cfg.name));
    ui.label(egui::RichText::new(label).small());
    changed
}
