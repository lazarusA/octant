//! The hover's composite rows: one per channel, its band and raw value at the
//! hovered pixel, read from the plot's block window or a detail tile's.

use std::sync::Arc;

use super::composite::{CompositeKind, CompositeLabels, composite_labels};
use crate::app::OctantApp;
use crate::data::slicing::CompositeProbe;
use crate::data::{DatasetMetadata, VariableInfo};
use crate::i18n::{HoverKey, tr};
use crate::ui::hover::field::HoverField;

/// One row per composite channel at composite pixel `(px, py)`: `R  NIR 0.312` for band
/// mappings, `DAPI  1234` for overlays.
pub fn composite_fields(
    app: &OctantApp,
    labels: &CompositeLabels,
    units: &str,
    pixel: (usize, usize),
) -> Vec<HoverField> {
    let probe = app.layers.base.data.composite_probe.as_ref();
    probe_fields(probe, labels, units, pixel)
}

/// [`composite_fields`] read from `probe` (the plot's or a detail tile's).
fn probe_fields(
    probe: Option<&CompositeProbe>,
    labels: &CompositeLabels,
    units: &str,
    (px, py): (usize, usize),
) -> Vec<HoverField> {
    let raw = |band: usize| probe.and_then(|p| p.sample(band, px, py));
    labels
        .channels
        .iter()
        .map(|c| {
            let mut value = String::with_capacity(32);
            if let Some(name) = &c.band_name {
                value.push_str(name);
                value.push(' ');
            }
            write_raw(&mut value, raw(c.band), units);
            HoverField::new(Arc::clone(&c.label), value)
        })
        .collect()
}

/// Appends a channel's raw value (with `units`), or "No data", to `out`.
fn write_raw(out: &mut String, raw: Option<f32>, units: &str) {
    let Some(value) = raw else {
        out.push_str(tr(HoverKey::NoData));
        return;
    };
    let mut buf = [0u8; 32];
    out.push_str(crate::ui::hover::card::HoverValue::from_raw(value, None).format(&mut buf));
    if !units.is_empty() && !value.is_nan() {
        out.push(' ');
        out.push_str(units);
    }
}

/// The composite's name, after prepending one row per channel (its band and raw value at
/// `pixel`, or where a detail tile is drawn at `tile`'s cell of its window) to `entries`
/// for flat 2D composites; `None` without a composite.
#[allow(clippy::too_many_arguments)]
pub fn composite_rows(
    app: &OctantApp,
    ctx: &egui::Context,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    units: &str,
    pixel: Option<(usize, usize)>,
    tile: Option<(&CompositeProbe, (usize, usize))>,
    entries: &mut Vec<HoverField>,
) -> Option<CompositeKind> {
    if !app.layers.base.composite.enabled {
        return None;
    }
    let labels = composite_labels(app, ctx, meta, var);
    let base = app.layers.base.data.composite_probe.as_ref();
    let at = match (pixel, tile) {
        (Some(_), Some((probe, cell))) => Some((probe, cell)),
        (Some(pixel), None) => base.map(|p| (p, pixel)),
        (None, _) => None,
    };
    if let Some((probe, cell)) = at {
        entries.splice(0..0, probe_fields(Some(probe), &labels, units, cell));
    }
    Some(labels.kind)
}
