//! Where a detail tile's cells and an irregular grid's values land on the
//! axes.

use super::axis_cells::{tile_cells, values};
use crate::ui::axes::{CellTicks, Lattice};

/// A 4 x 8 tile over the second quarter of x and the bottom half of the quad.
const UV: [f32; 4] = [0.25, 0.5, 0.5, 1.0];
const DOMS: ((f64, f64), (f64, f64)) = ((0.0, 100.0), (-50.0, 50.0));

#[test]
fn a_tile_sits_where_it_is_drawn_on_x() {
    let (x, _) = tile_cells(UV, (4, 8), DOMS, false);
    assert_eq!(x, Lattice::cells(25.0, 50.0, 4));
}

#[test]
fn a_tile_low_on_the_quad_covers_the_low_y_values() {
    // y grows upward: the quad's bottom half is the domain's lower half.
    let (_, y) = tile_cells(UV, (4, 8), DOMS, false);
    assert_eq!(y, Lattice::cells(-50.0, 0.0, 8));
}

#[test]
fn a_tile_low_on_a_raster_covers_the_high_y_values() {
    // Rows run down (OME-Zarr): the quad's bottom half is the upper half.
    let (_, y) = tile_cells(UV, (4, 8), DOMS, true);
    assert_eq!(y, Lattice::cells(0.0, 50.0, 8));
}

#[test]
fn irregular_values_tick_only_when_they_span_the_axis() {
    let lat = [80.0, 30.0, -9.0, -75.0];
    assert!(matches!(values(&lat, (-75.0, 80.0)), CellTicks::Values(_)));
    assert!(matches!(values(&lat, (80.0, -75.0)), CellTicks::Values(_)));
    // Coordinates shifted against the axis (another unit or a wrap) would
    // put ticks off their cells.
    assert!(matches!(values(&lat, (-90.0, 90.0)), CellTicks::None));
    assert!(matches!(values(&[], (-75.0, 80.0)), CellTicks::None));
}
