//! The surface and sphere hovers under the slicer's height cut: blocks it
//! cuts away (by their value, as the shaders do) are never picked.

use super::raycast_tests::{app, matrix, sphere_round_trip, surface_round_trip};
use crate::app::{SliceMode, Slicer};
use crate::data::CoordinateGrid;

/// Keeps values whose height is at least half the color range (0..100).
fn above_half() -> Slicer {
    Slicer {
        mode: SliceMode::Box,
        lo: [0.0, 0.0, 0.5],
        ..Slicer::default()
    }
}

#[test]
fn hovers_skip_blocks_the_height_cut_removes() {
    // Low blocks (20) on the left, tall ones (80) on the right.
    let m = matrix(12, 8, CoordinateGrid::GlobalRegular, |x, _| {
        if x < 6 { 20.0 } else { 80.0 }
    });
    // Seen from straight above, so no block hides behind a neighbor.
    let mut surface = app(std::f32::consts::FRAC_PI_2, 0.0);
    surface.plot_configs.mesh.surface_mode = 2;
    surface.plot_configs.mesh.surface_displacement = 0.2;
    let mut sphere = app(0.3, 0.0);
    sphere.plot_configs.mesh.sphere_mode = 3;
    sphere.plot_configs.mesh.sphere_displacement = 0.3;
    for app in [&mut surface, &mut sphere] {
        app.plot_configs.slice = above_half();
    }
    // Cells beside longitude 0, which faces the camera on the globe.
    let tall = (6, 4);
    assert_eq!(surface_round_trip(&surface, &m, tall.0, tall.1), Some(tall));
    assert_eq!(sphere_round_trip(&sphere, &m, tall.0, tall.1), Some(tall));
    for low in [(5, 4), (4, 3)] {
        assert_ne!(surface_round_trip(&surface, &m, low.0, low.1), Some(low));
        assert_ne!(sphere_round_trip(&sphere, &m, low.0, low.1), Some(low));
    }
}
