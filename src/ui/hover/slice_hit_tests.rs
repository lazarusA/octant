//! The volume hover under the slicer: rays start at its box and, in the
//! planes mode, read the nearest plane, as the shaders draw them.

use super::camera::Ray3D;
use super::raycast_volume::VolumeSampler;
use super::slice_hit::cut_bounds;
use crate::app::{OctantApp, SliceMode, Slicer};

const N: usize = 10;

/// A ray along -z through the middle of a unit box of `N`³ voxels of 5.
fn hit(slice: Slicer) -> Option<(usize, usize, usize, f32)> {
    let mut app = OctantApp::default();
    app.layers.base.color.range_min = 0.0;
    app.layers.base.color.range_max = 10.0;
    app.plot_configs.slice = slice;
    let values = vec![5.0; N * N * N];
    let sampler = VolumeSampler {
        width: N,
        height: N,
        depth: N,
        values: &values,
        shift_x: 0,
        shift_y: 0,
        shift_z: 0,
        layer: None,
        drawn: None,
    };
    let ray = Ray3D {
        origin: [0.01, 0.01, 5.0],
        dir: [0.0, 0.0, -1.0],
    };
    sampler.march_ray(&app, &ray, (1.0, 1.0, 1.0), true)
}

#[test]
fn rays_start_at_the_slicers_box() {
    let whole = hit(Slicer::default()).map(|h| h.2);
    assert_eq!(whole, Some(0), "the front plane first");
    let cut = Slicer {
        mode: SliceMode::Box,
        lo: [0.0, 0.0, 0.5],
        ..Slicer::default()
    };
    let front = hit(cut).map(|h| h.2).unwrap_or(0);
    assert!((4..=5).contains(&front), "the cut face first: {front}");
    let (lo, hi) = cut_bounds(&cut, (1.0, 2.0, 1.0));
    assert_eq!((lo, hi), ([-0.5, -1.0, -0.5], [0.5, 1.0, 0.0]));
}

#[test]
fn planes_are_read_where_the_ray_meets_them() {
    let mut planes = Slicer::default();
    planes.set_mode(SliceMode::Planes);
    planes.planes = [None, None, Some(0.75)];
    // y runs up: just above the middle is row 4 (rows run top down).
    assert_eq!(hit(planes), Some((5, 4, 7, 5.0)));
    planes.planes = [None; 3];
    assert_eq!(hit(planes), None, "no plane, nothing drawn");
}
