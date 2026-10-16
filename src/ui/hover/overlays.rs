//! Each drawn overlay's reading at the hovered cell of the base layer's grid,
//! for the hover card's layer rows.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::app::overlays::MAX_OVERLAYS;
use crate::i18n::{HoverKey, tr};
use crate::plots::PlotType;
use crate::ui::hover::card::LayerValue;
use crate::ui::hover::composite::CompositeKind;
use crate::ui::hover::detail::tile_reading;
use crate::ui::hover::entries::resolve_variable_units;
use crate::ui::hover::labels::layer_reading;
use crate::utils::colormap::evaluate_color_cpu;

/// Where the hover reads overlays: the base layer's cell and the point on
/// the plotted quad (`0..1` from its left and top edges).
#[derive(Debug, Clone, Copy)]
pub struct HoverAt {
    pub cell: (usize, usize),
    pub norm: (f32, f32),
}

/// Fills `out` with each drawn overlay's value at `at`, topmost first;
/// returns how many it wrote. Overlays share the base grid
/// (`Alignment::SameGrid`), so the base cell indexes them alike; an
/// overlay's detail tile under the point is read instead.
pub fn overlay_values<'a>(
    app: &'a OctantApp,
    at: HoverAt,
    out: &mut [LayerValue<'a>; MAX_OVERLAYS],
) -> usize {
    let drawn = app.layers.overlays().iter().rev().filter(|l| l.is_drawn());
    let mut count = 0;
    for (slot, layer) in out.iter_mut().zip(drawn) {
        let tile = layer.data.detail.tile.as_ref();
        let raw = tile
            .and_then(|t| tile_reading(t, at.norm.0, at.norm.1))
            .map_or_else(|| cell_value(layer, at.cell), |r| r.value);
        let var = layer.selection().variable_info();
        let name = var.map_or(tr(HoverKey::FallbackOverlay), |v| v.leaf_name());
        let units = resolve_variable_units(var);
        let swatch = || evaluate_color_cpu(raw, &app.get_color_params(layer));
        let composite = composite_kind(app, layer);
        let (value, swatch, units) = layer_reading(layer, raw, composite, units, swatch);
        *slot = LayerValue {
            name,
            value,
            units,
            swatch,
        };
        count += 1;
    }
    count
}

/// The overlay rows of the hover card, written into `out`: each drawn
/// overlay's reading at `at` on the heatmap, since overlays share the
/// heatmap's grid; none on other plots.
pub fn hover_rows<'a, 'o>(
    app: &'a OctantApp,
    plot_type: PlotType,
    at: Option<HoverAt>,
    out: &'o mut [LayerValue<'a>; MAX_OVERLAYS],
) -> &'o [LayerValue<'a>] {
    let count = match at {
        Some(at) if plot_type == PlotType::Heatmap => overlay_values(app, at, out),
        _ => 0,
    };
    out.get(..count).unwrap_or_default()
}

/// The kind of `layer`'s composite, while one is drawn: a channel overlay,
/// CMYK inks or bands drawn as RGB.
fn composite_kind(app: &OctantApp, layer: &Layer) -> Option<CompositeKind> {
    let id = layer.id();
    let kind = if !layer.composite.channel_configs.is_empty() && !app.layer_is_geotiff(id) {
        CompositeKind::Overlay
    } else if app.layer_is_cmyk(id) {
        CompositeKind::Cmyk
    } else {
        CompositeKind::Rgb
    };
    layer.composite.enabled.then_some(kind)
}

/// `layer`'s value at cell `(px, py)` of its full-resolution grid, NaN when
/// it has no data there.
fn cell_value(layer: &Layer, (px, py): (usize, usize)) -> f32 {
    // The matrix is the full-resolution data, also under a pyramid.
    let value = match &layer.data.matrix {
        Some(matrix) if px < matrix.width => matrix.values.get(py * matrix.width + px),
        _ => None,
    };
    value.copied().unwrap_or(f32::NAN)
}
