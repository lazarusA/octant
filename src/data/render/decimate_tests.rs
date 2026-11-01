//! Thinning matrices for meshes.

use std::sync::Arc;

use super::MatrixData;
use super::decimate::{drawn_index, sample_index, stride_for};
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
fn thinned_cells_and_coordinates_are_the_middle_of_each_drawn_span() {
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
    // x: 0, 2, 4 (3 cells drawn over 5); y: 1, 3.
    assert_eq!(sample_index(2, 2, 5), 4);
    assert_eq!(d.values, vec![5.0, 7.0, 9.0, 15.0, 17.0, 19.0]);
    let CoordinateGrid::Irregular1D {
        coords_x, coords_y, ..
    } = &d.grid
    else {
        panic!("irregular grid kept");
    };
    assert_eq!(&coords_x[..], &[0.0, 3.0, 15.0]);
    assert_eq!(&coords_y[..], &[20.0, 40.0]);
    assert!(m.decimated(1).is_none(), "nothing to thin");
}

#[test]
fn thinned_planes_keep_every_plane_and_the_middle_of_each_drawn_span() {
    // 5 x 4 x 3 voxels numbered in order; stride 2 keeps x 0, 2, 4 and y 1, 3.
    let values: Vec<f32> = (0..60).map(|i| i as f32).collect();
    let v = crate::data::VolumeData::new(5, 4, 3, values, 0.0, 59.0, String::new());
    let plane = |z: f32| [5.0, 7.0, 9.0, 15.0, 17.0, 19.0].map(|c| c + 20.0 * z);
    assert_eq!(v.thinned_planes(2, 1..3), [plane(1.0), plane(2.0)].concat());
}

#[test]
fn each_drawn_cell_shows_the_cell_under_its_middle() {
    // Drawn cells spread evenly over the axis, as regular grids and point
    // clouds draw them: 1951 by 50 draws 40, each over 48.8 cells.
    for (n, k) in [
        (1951_usize, 50_usize),
        (5, 2),
        (8, 2),
        (1001, 2),
        (7, 3),
        (2, 2),
        (9, 1),
    ] {
        let w = n.div_ceil(k);
        for i in 0..w {
            let x = sample_index(i, k, n);
            let middle = (i as f64 + 0.5) * n as f64 / w as f64;
            assert!(
                (x as f64 + 0.5 - middle).abs() <= 0.5,
                "{n} by {k}: drawn {i} shows {x}, its middle {middle}"
            );
            assert_eq!(drawn_index(x, k, n), i, "{n} by {k}: {x} back to {i}");
        }
        assert_eq!(drawn_index(n - 1, k, n), w - 1);
    }
    assert_eq!(sample_index(20, 50, 1951), 999);
}
