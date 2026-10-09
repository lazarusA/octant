//! Each drawn overlay's reading at the hovered cell of the base layer's grid,
//! for the hover card's layer rows.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::app::overlays::MAX_OVERLAYS;
use crate::ui::hover::card::{HoverValue, LayerValue};
use crate::ui::hover::entries::resolve_variable_units;
use crate::utils::colormap::evaluate_color_cpu;

/// Fills `out` with each drawn overlay's value at base cell `cell`, topmost
/// first; returns how many it wrote. Overlays share the base grid
/// (`Alignment::SameGrid`), so the cell indexes them alike.
pub fn overlay_values<'a>(
    app: &'a OctantApp,
    cell: (usize, usize),
    out: &mut [LayerValue<'a>; MAX_OVERLAYS],
) -> usize {
    let drawn = app.layers.overlays().iter().rev().filter(|l| l.is_drawn());
    let mut count = 0;
    for (slot, layer) in out.iter_mut().zip(drawn) {
        let raw = cell_value(layer, cell);
        let var = layer.selection().variable_info();
        *slot = LayerValue {
            name: var.map_or("overlay", |v| v.leaf_name()),
            value: HoverValue::from_raw(raw, None),
            units: resolve_variable_units(var),
            swatch: evaluate_color_cpu(raw, &app.get_color_params(layer)),
        };
        count += 1;
    }
    count
}

/// `layer`'s value at cell `(px, py)` of its full-resolution grid, NaN when
/// it has no data there.
fn cell_value(layer: &Layer, (px, py): (usize, usize)) -> f32 {
    let data = &layer.data;
    let value = match (&data.pyramid, &data.matrix) {
        (Some(pyramid), _) => pyramid
            .levels
            .first()
            .and_then(|level| level.values.get(py * pyramid.original_width + px)),
        (None, Some(matrix)) if px < matrix.width => matrix.values.get(py * matrix.width + px),
        _ => None,
    };
    value.copied().unwrap_or(f32::NAN)
}
