//! RGB, CMYK, and Multi-Channel overlay controls for 2D settings panel.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::slicing::{ChannelColorConfig, CompositeProbe};
use crate::i18n::{SettingsKey as K, VariablesKey, tr};
use crate::ui::hover::composite::composite_labels;
use crate::ui::icons::{Icon, IconSize, ToolbarButton};
use crate::utils::stack_str;

/// Auto contrast: the quantiles of each channel's shown values its window spans.
const AUTO_LOW: f64 = 0.001;
const AUTO_HIGH: f64 = 0.999;

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
            ui.label(egui::RichText::new(tr(K::CmykAutoMapped)).small().weak());
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
    let (label, tooltip) = (tr(label), tr(tooltip));
    // The band names cache follows the base layer (the hover card's).
    let mut label_buf = [0u8; 160];
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
fn toggle_text(has_mc: bool, is_cmyk: bool) -> (K, K) {
    if has_mc {
        (K::MultiChannelOverlay, K::MultiChannelOverlayHover)
    } else if is_cmyk {
        (K::CmykComposite, K::CmykCompositeHover)
    } else {
        (K::RgbComposite, K::RgbCompositeHover)
    }
}

/// Layer `id`'s selected band range, or every band without a band dimension
/// and for overlays, which have no band slider (`fit_overlay_channels`).
pub(super) fn get_selected_channel_range(app: &OctantApp, id: LayerId) -> (usize, usize) {
    match app.layer_channel_dim(id) {
        Some(c_idx) if id == LayerId::BASE => app.layer_dim_range(id, c_idx),
        _ => (0, usize::MAX),
    }
}

/// Layer `id`'s multi-channel overlay: the channel buttons, then one row per
/// channel with its display settings behind a chevron; reloads the layer
/// when a channel settles on a change.
fn show_multichannel_controls(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    let (start, end) = get_selected_channel_range(app, id);
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    // The 2D composite's channel values, for auto contrast.
    let probe = layer.data.composite_probe.as_ref();
    // Configs are indexed by channel; only the selected range is listed.
    let all = &mut layer.composite.channel_configs;
    let end = end.saturating_add(1).min(all.len());
    let configs = &mut all[start.min(end)..end];
    let mut changed = channel_buttons(ui, configs, probe);
    egui::ScrollArea::vertical()
        .max_height(180.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for cfg in configs.iter_mut() {
                changed |= channel_entry(ui, cfg, id);
            }
        });
    if changed {
        app.load_layer_block(id);
    }
}

/// Show all channels, none, or set every window from the shown values
/// (Auto, while the composite's values are known). Whether any changed.
fn channel_buttons(
    ui: &mut egui::Ui,
    configs: &mut [ChannelColorConfig],
    probe: Option<&CompositeProbe>,
) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(tr(K::Channels)).small().strong());
        for (label, visible) in [(K::ChannelsAll, true), (K::ChannelsNone, false)] {
            if ui.small_button(tr(label)).clicked() {
                configs.iter_mut().for_each(|cfg| cfg.visible = visible);
                changed = true;
            }
        }
        let auto = ui
            .add_enabled(
                probe.is_some(),
                egui::Button::new(tr(K::ChannelsAuto)).small(),
            )
            .on_hover_text(tr(K::ChannelsAutoHover))
            .on_disabled_hover_text(tr(K::ChannelsAutoUnavailable));
        if auto.clicked()
            && let Some(probe) = probe
        {
            for cfg in configs.iter_mut() {
                if let Some(window) = probe.channel_quantiles(cfg.index, AUTO_LOW, AUTO_HIGH) {
                    cfg.window = Some(window);
                    changed = true;
                }
            }
        }
    });
    changed
}

/// One channel's row and, when its chevron is open, its display settings.
/// Whether the layer should reload.
fn channel_entry(ui: &mut egui::Ui, cfg: &mut ChannelColorConfig, id: LayerId) -> bool {
    let open_id = egui::Id::unique(("mc_channel_open", id, cfg.index));
    let mut open = ui.data(|d| d.get_temp::<bool>(open_id).unwrap_or(false));
    let mut changed = false;
    ui.horizontal(|ui| {
        changed = channel_row(ui, cfg, id);
        let chevron = if open {
            Icon::ChevronDown
        } else {
            Icon::ChevronRight
        };
        let toggle = ToolbarButton::new(chevron, tr(K::ChannelSettingsHover))
            .compact(true)
            .icon_size(IconSize::Xs);
        if ui.add(toggle).clicked() {
            open = !open;
            ui.data_mut(|d| d.insert_temp(open_id, open));
        }
    });
    if open {
        changed |= super::channel_display::channel_detail(ui, cfg, id);
    }
    changed
}

/// One channel of layer `id`'s multi-channel overlay: visibility, tint and
/// "N: name". Whether the channel changed.
fn channel_row(ui: &mut egui::Ui, cfg: &mut ChannelColorConfig, id: LayerId) -> bool {
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
    .tooltip(tr(K::ChannelTintHover))
    .show(ui);
    if color != initial {
        let to_u8 = |c: f32| (c * 255.0).round().clamp(0.0, 255.0) as u8;
        cfg.color_rgb = [to_u8(color[0]), to_u8(color[1]), to_u8(color[2])];
        changed = true;
    }
    let mut buf = [0u8; 96];
    let number = cfg.index + 1;
    let label = match cfg.name.trim() {
        "" => crate::i18n::fill(&mut buf, tr(VariablesKey::ChannelN), &[&number]),
        name => stack_str(&mut buf, format_args!("{number}: {name}")),
    };
    ui.label(egui::RichText::new(label).small());
    changed
}
