//! One channel's display settings in a multi-channel overlay: its window,
//! intensity curve and inversion (OMERO `window`, `family`, `inverted`).

use crate::app::layers::LayerId;
use crate::data::ome::ChannelFamily;
use crate::data::slicing::ChannelColorConfig;
use crate::i18n::{SettingsKey as K, tr};
use crate::ui::number_input::LocalizedInput;

const FAMILIES: [(ChannelFamily, K); 4] = [
    (ChannelFamily::Linear, K::CurveLinear),
    (ChannelFamily::Logarithmic, K::CurveLogarithmic),
    (ChannelFamily::Exponential, K::CurveExponential),
    (ChannelFamily::Polynomial, K::CurvePolynomial),
];

/// The window, curve and inversion of channel `cfg` of layer `id`. Whether
/// anything changed.
pub(super) fn channel_detail(ui: &mut egui::Ui, cfg: &mut ChannelColorConfig, id: LayerId) -> bool {
    let mut changed = false;
    ui.indent(("mc_channel_detail", id, cfg.index), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(tr(K::ChannelWindow)).small())
                .on_hover_text(tr(K::ChannelWindowHover));
            changed |= window_inputs(ui, cfg);
        });
        ui.horizontal_wrapped(|ui| {
            changed |= ui
                .checkbox(&mut cfg.display.inverted, tr(K::ChannelInvert))
                .changed();
            changed |= curve_select(ui, cfg, id);
        });
    });
    changed
}

/// Start and end of the window, within the values it may span (OMERO
/// `min`..`max`, else the window itself).
fn window_inputs(ui: &mut egui::Ui, cfg: &mut ChannelColorConfig) -> bool {
    let (lo, hi) = cfg.display.range.or(cfg.window).unwrap_or((0.0, 1.0));
    let (mut start, mut end) = cfg.window.unwrap_or((lo, hi));
    let speed = ((hi - lo).abs() / 200.0).max(1e-4);
    let mut changed = ui
        .add(
            egui::DragValue::new(&mut start)
                .localized()
                .speed(speed)
                .range(lo.min(end)..=end),
        )
        .changed();
    changed |= ui
        .add(
            egui::DragValue::new(&mut end)
                .localized()
                .speed(speed)
                .range(start..=hi.max(start)),
        )
        .changed();
    if changed {
        cfg.window = Some((start, end));
    }
    changed
}

/// The curve family and, for non-linear curves, its coefficient.
fn curve_select(ui: &mut egui::Ui, cfg: &mut ChannelColorConfig, id: LayerId) -> bool {
    let mut changed = false;
    let name = |family: ChannelFamily| {
        FAMILIES
            .iter()
            .find(|(f, _)| *f == family)
            .map_or("", |(_, key)| tr(*key))
    };
    ui.label(egui::RichText::new(tr(K::ChannelCurve)).small());
    egui::ComboBox::from_id_salt(("mc_channel_curve", id, cfg.index))
        .selected_text(name(cfg.display.family))
        .show_ui(ui, |ui| {
            for (family, key) in FAMILIES {
                changed |= ui
                    .selectable_value(&mut cfg.display.family, family, tr(key))
                    .changed();
            }
        });
    if cfg.display.family != ChannelFamily::Linear {
        changed |= ui
            .add(
                egui::DragValue::new(&mut cfg.display.coefficient)
                    .localized()
                    .speed(0.05)
                    .range(0.05..=10.0),
            )
            .on_hover_text(tr(K::ChannelCoefficientHover))
            .changed();
    }
    changed
}
