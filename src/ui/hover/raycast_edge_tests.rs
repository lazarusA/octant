//! Surface and sphere hovers where the drawn geometry is not a plain
//! heightfield: column walls, gaps between steps, faces seen from below,
//! regions that cover only part of the globe, and grids large enough to
//! stretch the march.

use super::raycast_tests::{app, matrix, rect, sphere_round_trip, surface_round_trip};
use super::*;
use crate::data::{CoordinateGrid, MatrixData};
use crate::utils::height::{HeightScale, surface_height};

/// The cell picked under where world point `p` appears on screen.
fn pick_surface(
    app: &crate::app::OctantApp,
    m: &MatrixData,
    p: [f32; 3],
) -> Option<(usize, usize)> {
    let camera = Camera3D::from_app(app, rect());
    let screen = camera.project_point(p)?;
    let (nx, ny, _) = raycast_surface(app, m, &camera, screen)?;
    Some(m.grid.find_cell_from_norm(nx, ny, m.width, m.height))
}

fn height(app: &crate::app::OctantApp, val: f32) -> f32 {
    let mesh = &app.plot_configs.mesh;
    let color = &app.layers.base.color;
    let scale = HeightScale {
        cmin: color.range_min,
        cmax: color.range_max,
        composite: false,
        flip: false,
    };
    surface_height(val, &scale, mesh.surface_mode, mesh.surface_displacement)
}

#[test]
fn block_wall_at_the_grid_edge_is_picked_where_the_ray_enters() {
    // A tall column on the west edge seen from the west: the ray enters the
    // bounding box through its outer wall.
    let m = matrix(8, 8, CoordinateGrid::GlobalRegular, |x, y| {
        if (x, y) == (0, 4) { 100.0 } else { 0.0 }
    });
    let mut app = app(0.15, std::f32::consts::FRAC_PI_2);
    app.plot_configs.mesh.surface_mode = 2;
    app.plot_configs.mesh.surface_displacement = 1.0;
    let z = 2.0 * 4.5 / 8.0 - 1.0;
    let wall = [-1.0, 0.5 * height(&app, 100.0), z];
    assert_eq!(pick_surface(&app, &m, wall), Some((0, 4)));
}

#[test]
fn steps_show_what_lies_behind_the_gap_under_a_raised_step() {
    // Steps are flat patches: a low ray passes under the raised one and
    // lands on the low step behind it.
    let m = matrix(8, 8, CoordinateGrid::GlobalRegular, |x, y| {
        if (x, y) == (5, 4) { 100.0 } else { 0.0 }
    });
    let mut app = app(0.15, std::f32::consts::FRAC_PI_2);
    app.plot_configs.mesh.surface_mode = 1;
    app.plot_configs.mesh.surface_displacement = 1.0;
    assert_eq!(surface_round_trip(&app, &m, 6, 4), Some((6, 4)));
}

#[test]
fn negative_block_is_picked_from_below() {
    // A range spanning zero hangs negative columns below the zero plane;
    // seen from below, the ray meets the column's bottom face.
    let m = matrix(16, 16, CoordinateGrid::GlobalRegular, |x, y| {
        if (x, y) == (8, 8) { -100.0 } else { 0.0 }
    });
    let mut app = app(-0.6, 0.3);
    app.layers.base.color.range_min = -100.0;
    app.plot_configs.mesh.surface_mode = 2;
    app.plot_configs.mesh.surface_displacement = 1.0;
    let (x, z) = (2.0 * 8.5 / 16.0 - 1.0, 2.0 * 8.5 / 16.0 - 1.0);
    let bottom = [x, height(&app, -100.0), z];
    assert_eq!(pick_surface(&app, &m, bottom), Some((8, 8)));
}

#[test]
fn spike_on_a_large_grid_is_not_stepped_over() {
    // A long strip seen lengthwise: the ray crosses thousands of cells, more
    // than a fixed sample budget covers in half-cell steps.
    let (w, h) = (20_000, 200);
    let m = matrix(w, h, CoordinateGrid::GlobalRegular, |x, y| {
        if (x, y) == (10_000, 100) { 100.0 } else { 0.0 }
    });
    // Just above the ground and nearly level, aiming at the spike's wall
    // near its base, the ray runs the strip's whole length in the box.
    let mut app = app(0.004, std::f32::consts::FRAC_PI_2);
    // Placed so a 8192-sample budget (a 0.0125 step) falls on either side
    // of the 0.01-wide spike.
    app.nav.sphere_zoom = 2.505;
    app.plot_configs.mesh.surface_mode = 2;
    app.plot_configs.mesh.surface_displacement = 1.0;
    let z = 2.0 * 100.5 / 200.0 - 1.0;
    assert_eq!(
        pick_surface(&app, &m, [0.0, 0.0099, z]),
        Some((10_000, 100))
    );
}

#[test]
fn regional_sphere_hovers_only_its_region() {
    let grid = CoordinateGrid::RegionalRegular {
        lon_bounds: (-20.0, 40.0),
        lat_bounds: (10.0, 50.0),
    };
    let m = matrix(12, 8, grid, |x, y| ((x * 7 + y * 13) % 100) as f32);
    for mode in 0..4 {
        let mut app = app(0.0, 0.0);
        app.plot_configs.mesh.sphere_mode = mode;
        app.plot_configs.mesh.sphere_displacement = 0.3;
        // Just south of the region the ray enters the globe and rises into
        // the region's latitudes below its surface: nothing drawn is hit.
        let camera = Camera3D::from_app(&app, rect());
        let south = 5f32.to_radians();
        let screen = camera.project_point([0.0, south.sin(), south.cos()]);
        let picked = screen.and_then(|p| raycast_sphere(&app, &m, &camera, p));
        assert_eq!(picked, None, "mode {mode}");
        assert_eq!(
            sphere_round_trip(&app, &m, 3, 6),
            Some((3, 6)),
            "mode {mode}"
        );
    }
}

#[test]
fn mesh_hits_are_cached_until_pointer_view_or_data_change() {
    use crate::plots::PlotType;
    let mut app = app(0.5, 0.0);
    let (r, p) = (rect(), rect().center());
    assert_eq!(
        hit_cache::mesh_hit_key(&app, r, p),
        None,
        "heatmaps are not marched"
    );
    app.selected.plot_type = PlotType::Surface;
    let key = hit_cache::mesh_hit_key(&app, r, p);
    assert!(key.is_some());
    assert_eq!(hit_cache::mesh_hit_key(&app, r, p), key);
    assert_ne!(
        hit_cache::mesh_hit_key(&app, r, p + egui::vec2(1.0, 0.0)),
        key
    );
    app.nav.sphere_rotation_y = 0.1;
    let turned = hit_cache::mesh_hit_key(&app, r, p);
    assert_ne!(turned, key);
    app.layers.base.data.touch_matrix();
    assert_ne!(hit_cache::mesh_hit_key(&app, r, p), turned);
}
