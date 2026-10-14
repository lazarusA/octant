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
/// the layer should reload: a dragged value reloads at most every
/// [`DRAG_RELOAD_SECS`] and once more when its drag stops (see [`settled`]).
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

/// The shortest time between reloads while a value is dragged.
const DRAG_RELOAD_SECS: f64 = 0.1;

/// Whether an edit of `response`'s value should reload the layer: when its
/// drag stops, when it changed without a drag (typing, arrow keys), and
/// while dragged at most every [`DRAG_RELOAD_SECS`], so the image follows
/// the drag without re-projecting the composite every frame.
fn settled(ui: &egui::Ui, response: &egui::Response) -> bool {
    if response.drag_stopped() || (response.changed() && !response.dragged()) {
        return true;
    }
    if !response.changed() {
        return false;
    }
    let now = ui.input(|i| i.time);
    let id = response.id.with("last_drag_reload");
    if ui
        .data(|d| d.get_temp::<f64>(id))
        .is_some_and(|last| now - last < DRAG_RELOAD_SECS)
    {
        return false;
    }
    ui.data_mut(|d| d.insert_temp(id, now));
    true
}

/// A localized input of `value` within `range`.
fn input(value: &mut f32, speed: f32, range: std::ops::RangeInclusive<f32>) -> impl egui::Widget {
    egui::DragValue::new(value)
        .localized()
        .speed(speed)
        .range(range)
}

/// Start and end of the window, within the values it may span (OMERO
/// `min`..`max`, else the window itself). Whether the layer should reload.
fn window_inputs(ui: &mut egui::Ui, cfg: &mut ChannelColorConfig) -> bool {
    let (lo, hi) = cfg.display.range.or(cfg.window).unwrap_or((0.0, 1.0));
    let (mut start, mut end) = cfg.window.unwrap_or((lo, hi));
    let speed = ((hi - lo).abs() / 200.0).max(1e-4);
    let a = ui.add(input(&mut start, speed, lo.min(end)..=end));
    let b = ui.add(input(&mut end, speed, start..=hi.max(start)));
    if a.changed() || b.changed() {
        cfg.window = Some((start, end));
    }
    settled(ui, &a) || settled(ui, &b)
}

/// The curve family and, for the curves that use one (polynomial and
/// exponential; OMERO's logarithmic ignores it), its coefficient. Whether
/// the layer should reload.
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
    if matches!(
        cfg.display.family,
        ChannelFamily::Polynomial | ChannelFamily::Exponential
    ) {
        let response = ui
            .add(input(&mut cfg.display.coefficient, 0.05, 0.05..=10.0))
            .on_hover_text(tr(K::ChannelCoefficientHover));
        changed |= settled(ui, &response);
    }
    changed
}
