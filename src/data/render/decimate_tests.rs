//! Thinning matrices for meshes.

use std::sync::Arc;

use super::MatrixData;
use super::decimate::{sample_index, stride_for};
use crate::data::coordinates::CoordinateGrid;

#[test]
fn the_stride_leaves_at_most_the_budget() {
    assert_eq!(stride_for(100, 100), 1);
    assert_eq!(stride_for(24_714_000, 1_036_800), 5);
    for (cells, budget) in [(10_001, 100), (7031 * 3515, 500_000), (3, 1)] {
        let k = stride_for(cells, budget);
        assert!(cells.div_ceil(k * k) <= budget, "{cells} by {k}");
    }
}

#[test]
fn thinned_cells_and_coordinates_are_the_middle_of_each_stride() {
    let (w, h) = (5, 4);
    let values: Vec<f32> = (0..w * h).map(|i| i as f32).collect();
    let grid = CoordinateGrid::Irregular1D {
        coords_x: Arc::from([0.0, 1.0, 3.0, 7.0, 15.0]),
        coords_y: Arc::from([10.0, 20.0, 30.0, 40.0]),
        lon_bounds: (0.0, 15.0),
        lat_bounds: (10.0, 40.0),
    };
    let m = MatrixData::new_with_grid(w, h, values, 0.0, 19.0, "m".into(), 1, grid);
    let d = m.decimated(2).expect("thinned");
    assert_eq!((d.width, d.height), (3, 2));
    // x: 1, 3, 4 (the last stride is short); y: 1, 3.
    assert_eq!(sample_index(2, 2, 5), 4);
    assert_eq!(d.values, vec![6.0, 8.0, 9.0, 16.0, 18.0, 19.0]);
    let CoordinateGrid::Irregular1D {
        coords_x, coords_y, ..
    } = &d.grid
    else {
        panic!("irregular grid kept");
    };
    assert_eq!(&coords_x[..], &[1.0, 7.0, 15.0]);
    assert_eq!(&coords_y[..], &[20.0, 40.0]);
    assert!(m.decimated(1).is_none(), "nothing to thin");
}
