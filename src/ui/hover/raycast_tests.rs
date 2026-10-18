//! Surface and sphere hovers pick the cell drawn under the cursor: a cell
//! center projected to the screen raycasts back to that cell.

use super::march::{Probe, Shape, Side, march};
use super::smooth::{bracket, bracket_wrapped, index_sample};
use super::*;
use crate::app::OctantApp;
use crate::data::{CoordinateGrid, MatrixData};
use egui::{Rect, pos2};

pub(super) fn rect() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0))
}

pub(super) fn matrix(
    w: usize,
    h: usize,
    grid: CoordinateGrid,
    value: impl Fn(usize, usize) -> f32,
) -> MatrixData {
    let values: Vec<f32> = (0..w * h).map(|i| value(i % w, i / w)).collect();
    MatrixData::new_with_grid(w, h, values, 0.0, 100.0, "test".to_string(), 1, grid)
}

pub(super) fn app(rotation_x: f32, rotation_y: f32) -> OctantApp {
    let mut app = OctantApp::default();
    app.layers.base.color.range_min = 0.0;
    app.layers.base.color.range_max = 100.0;
    app.nav.sphere_rotation_x = rotation_x;
    app.nav.sphere_rotation_y = rotation_y;
    app
}

/// Raycasts from where cell `(px, py)`'s drawn center appears on screen.
pub(super) fn surface_round_trip(
    app: &OctantApp,
    m: &MatrixData,
    px: usize,
    py: usize,
) -> Option<(usize, usize)> {
    let camera = Camera3D::from_app(app, rect());
    let screen = surface_target_pos(app, m, &camera, px, py)?;
    let (nx, ny, _) = raycast_surface(app, m, &camera, screen)?;
    Some(m.grid.find_cell_from_norm(nx, ny, m.width, m.height))
}

pub(super) fn sphere_round_trip(
    app: &OctantApp,
    m: &MatrixData,
    px: usize,
    py: usize,
) -> Option<(usize, usize)> {
    let camera = Camera3D::from_app(app, rect());
    let screen = sphere_target_pos(app, m, &camera, px, py)?;
    let (nx, ny, _) = raycast_sphere(app, m, &camera, screen)?;
    Some(m.grid.find_cell_from_norm(nx, ny, m.width, m.height))
}

#[test]
fn surface_hover_picks_every_cell_from_above() {
    // Rough heights seen from straight above, low enough that no cell's
    // center hides behind a neighbor in perspective.
    let m = matrix(12, 8, CoordinateGrid::GlobalRegular, |x, y| {
        ((x * 37 + y * 61) % 101) as f32
    });
    for mode in 0..3 {
        for flip in [false, true] {
            let mut app = app(std::f32::consts::FRAC_PI_2, 0.0);
            app.plot_configs.mesh.surface_mode = mode;
            app.plot_configs.mesh.surface_displacement = 0.2;
            app.plot_configs.mesh.surface_flip_height = flip;
            for py in 0..m.height {
                for px in 0..m.width {
                    let picked = surface_round_trip(&app, &m, px, py);
                    assert_eq!(picked, Some((px, py)), "mode {mode}, flip {flip}");
                }
            }
        }
    }
}

/// Panned to a corner and zoomed in, the hover still picks the cells drawn
/// under the cursor.
#[test]
fn surface_hover_follows_the_panned_camera() {
    let m = matrix(12, 8, CoordinateGrid::GlobalRegular, |x, y| {
        ((x * 37 + y * 61) % 101) as f32
    });
    let mut app = app(1.2, 0.4);
    app.plot_configs.mesh.surface_displacement = 0.2;
    app.nav.orbit_target = [-1.0, 0.0, -0.6];
    app.nav.sphere_zoom = 0.8;
    let camera = Camera3D::from_app(&app, rect());
    let mut seen = 0;
    for py in 0..m.height {
        for px in 0..m.width {
            let on_screen = surface_target_pos(&app, &m, &camera, px, py)
                .is_some_and(|pos| rect().shrink(4.0).contains(pos));
            if on_screen {
                seen += 1;
                assert_eq!(surface_round_trip(&app, &m, px, py), Some((px, py)));
            }
        }
    }
    assert!(
        (4..m.width * m.height / 2).contains(&seen),
        "{seen} cells shown"
    );
}

#[test]
fn surface_hover_picks_the_plateau_in_front_not_the_ground_behind() {
    // A plateau seen at a shallow angle covers the cells behind it; the old
    // pick, through the zero plane, landed on them.
    let m = matrix(16, 16, CoordinateGrid::GlobalRegular, |x, y| {
        if (7..=9).contains(&x) && (7..=9).contains(&y) {
            100.0
        } else {
            0.0
        }
    });
    for mode in 0..3 {
        let mut app = app(0.9, 0.3);
        app.plot_configs.mesh.surface_mode = mode;
        app.plot_configs.mesh.surface_displacement = 2.0;
        assert_eq!(
            surface_round_trip(&app, &m, 8, 8),
            Some((8, 8)),
            "mode {mode}"
        );
    }
}

#[test]
fn sphere_hover_picks_front_cells() {
    // Gentle relief so no front cell hides another.
    let m = matrix(36, 18, CoordinateGrid::GlobalRegular, |x, y| {
        50.0 + 40.0 * ((x as f32 * 0.4).sin() * (y as f32 * 0.5).cos())
    });
    for mode in 0..4 {
        let mut app = app(0.0, 0.0);
        app.plot_configs.mesh.sphere_mode = mode;
        app.plot_configs.mesh.sphere_displacement = 0.3;
        for py in 0..m.height {
            for px in 0..m.width {
                let (lon, lat) = m.grid.cell_center_lon_lat_rad(px, py, m.width, m.height);
                // Cells facing the camera, which sits on +z.
                if lat.cos() * lon.cos() < 0.8 {
                    continue;
                }
                let picked = sphere_round_trip(&app, &m, px, py);
                assert_eq!(picked, Some((px, py)), "mode {mode}");
            }
        }
    }
}

#[test]
fn regional_lookup_inverts_cell_centers() {
    let grid = CoordinateGrid::RegionalRegular {
        lon_bounds: (-20.0, 40.0),
        lat_bounds: (10.0, 50.0),
    };
    let (w, h) = (10, 6);
    for py in 0..h {
        for px in 0..w {
            let (lon, lat) = grid.cell_center_lon_lat_rad(px, py, w, h);
            assert_eq!(
                grid.find_cell_from_lon_lat_rad(lon, lat, w, h),
                Some((px, py))
            );
        }
    }
}

#[test]
fn smooth_vertices_sit_at_centers_and_stretch_to_edges() {
    let n = 4;
    let samples: Vec<f32> = (0..n).map(|j| index_sample(j, n)).collect();
    assert_eq!(samples, [0.0, 1.5, 2.5, 4.0]);
    let sample = |j| index_sample(j, n);
    assert_eq!(bracket(2.0, 1, n, sample), (1, 2, 0.5));
    let near = |(a, b, t): (usize, usize, f32), want: (usize, usize, f32)| {
        (a, b) == (want.0, want.1) && (t - want.2).abs() < 1e-6
    };
    assert!(near(bracket(1.0, 1, n, sample), (0, 1, 1.0 / 1.5)));
    assert!(near(bracket(3.9, 3, n, sample), (2, 3, 1.4 / 1.5)));
    // Wrapped: between the last center and the first, across the seam.
    let (a, b, t) = bracket_wrapped(0.25, n);
    assert_eq!((a, b), (3, 0));
    assert!((t - 0.75).abs() < 1e-6);
}

#[test]
fn march_catches_a_column_crossed_within_one_step() {
    // Above the column at one sample and below it at the next: the ray went
    // through its top, though no sample landed inside.
    let side = |t: f32| if t < 5.0 { Side::Above } else { Side::Below };
    let probe = |t: f32| Probe {
        cell: Some((0, 0)),
        side: side(t),
    };
    let t = march((0.0, 10.0), 4.0, 100, Shape::Columns, probe);
    assert!(t.is_some_and(|t| (t - 5.0).abs() < 1e-2));
}

#[test]
fn march_passes_under_patches_into_other_cells() {
    // Above cell 0's patch, then below cell 1's: steps have no walls.
    let probe = |t: f32| Probe {
        cell: Some((usize::from(t >= 5.0), 0)),
        side: if t < 5.0 { Side::Above } else { Side::Below },
    };
    assert_eq!(march((0.0, 10.0), 1.0, 100, Shape::Patches, probe), None);
    // A continuous surface is crossed there.
    let t = march((0.0, 10.0), 1.0, 100, Shape::Smooth, probe);
    assert!(t.is_some_and(|t| (t - 5.0).abs() < 1e-2));
}
