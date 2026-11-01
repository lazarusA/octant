//! Thinning matrices for meshes.

use std::sync::Arc;

use super::MatrixData;
use super::decimate::{sample_index, stride_for};
use crate::data::coordinates::CoordinateGrid;

#[test]
fn the_stride_leaves_at_most_the_budget() {
    assert_eq!(stride_for(10, 10, 100), 1);
    assert_eq!(stride_for(7030, 3515, 1_036_800), 5);
    // Square, narrow and odd grids, counted as `decimated` keeps them: an
    // 11585^2 grid at 2 keeps 5793^2 cells, past 2^25; a 1 x 70M slice
    // thins along one axis only.
    for (w, h, budget) in [
        (101, 99, 100),
        (7031, 3515, 500_000),
        (3, 1, 1),
        (11_585, 11_585, 1 << 25),
        (1, 70_000_000, 1 << 25),
        (1, 10_000_000, 1_036_800),
    ] {
        let k = stride_for(w, h, budget);
        let kept = w.div_ceil(k) * h.div_ceil(k);
        assert!(kept <= budget, "{w}x{h} by {k} keeps {kept}");
        if k > 1 {
            let more = w.div_ceil(k - 1) * h.div_ceil(k - 1);
            assert!(more > budget, "{w}x{h}: {} would fit", k - 1);
        }
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
