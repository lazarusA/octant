//! The gizmo places cells where the renderers and the hover do: regional
//! globes half a cell past their bounds, surfaces by their cells.

use super::space::Space;
use super::tests::{rect, screen};
use crate::data::{CoordinateGrid, MatrixData};
use crate::plots::PlotType;
use crate::ui::hover::camera::Camera3D;
use crate::ui::hover::raycast_sphere::sphere_target_pos;
use crate::ui::hover::raycast_surface::surface_target_pos;
use egui::pos2;

fn app_with(plot_type: PlotType, m: MatrixData) -> crate::app::OctantApp {
    let mut app = crate::app::OctantApp::default();
    app.layers.base.color.range_min = 0.0;
    app.layers.base.color.range_max = 100.0;
    app.nav.sphere_rotation_x = 0.2;
    app.selected.plot_type = plot_type;
    app.plot_configs.mesh.surface_displacement = 0.0;
    app.plot_configs.mesh.sphere_mode = 0;
    app.layers.base.data.matrix = Some(m);
    app
}

fn grid_matrix(w: usize, h: usize, grid: CoordinateGrid) -> MatrixData {
    let values = vec![50.0; w * h];
    MatrixData::new_with_grid(w, h, values, 0.0, 100.0, "t".to_string(), 1, grid)
}

/// Cell `(px, py)`'s center as the gizmo places it and as the hover does.
fn both_places(app: &crate::app::OctantApp, px: usize, py: usize) -> (egui::Pos2, egui::Pos2) {
    let Some(space) = Space::of(app, rect()) else {
        return (pos2(0.0, 0.0), pos2(1e9, 1e9));
    };
    let m = app.layers.base.data.matrix.as_ref();
    let (w, h) = m.map_or((1, 1), |m| (m.width, m.height));
    let f = [
        (px as f32 + 0.5) / w as f32,
        (py as f32 + 0.5) / h as f32,
        0.0,
    ];
    let camera = Camera3D::from_app(app, rect());
    let hover = m.and_then(|m| match app.selected.plot_type {
        PlotType::Sphere => sphere_target_pos(app, m, &camera, px, py),
        _ => surface_target_pos(app, m, &camera, px, py),
    });
    (screen(&space, f), hover.unwrap_or(pos2(1e9, 1e9)))
}

#[test]
fn the_gizmo_places_cells_where_the_hover_does() {
    let regional = CoordinateGrid::RegionalRegular {
        lon_bounds: (-20.0, 40.0),
        lat_bounds: (10.0, 50.0),
    };
    let cases = [
        (PlotType::Sphere, regional),
        (PlotType::Sphere, CoordinateGrid::GlobalRegular),
        (PlotType::Surface, CoordinateGrid::GlobalRegular),
    ];
    for (plot_type, grid) in cases {
        let app = app_with(plot_type, grid_matrix(12, 8, grid));
        for (px, py) in [(0, 0), (3, 6), (11, 7)] {
            let (gizmo, hover) = both_places(&app, px, py);
            assert!(
                gizmo.distance(hover) < 0.5,
                "{plot_type:?} cell ({px}, {py}): {gizmo:?} vs {hover:?}"
            );
        }
    }
}
