//! Surfaces and spheres of large 2D data draw a thinned grid, with a GPU.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::test_support::render_state;
use crate::data::MatrixData;

/// `n` x `n` cells built into the base layer's renderers on a 512 x 512 canvas.
fn built(n: usize) -> Option<OctantApp> {
    let mut app = OctantApp {
        wgpu_render_state: Some(render_state()?),
        ..Default::default()
    };
    app.layout.canvas_px = [512, 512];
    let values = (0..n * n).map(|i| i as f32).collect();
    let data = MatrixData::new(n, n, values, 0.0, (n * n) as f32, "m".into(), 1);
    app.rebuild_pipeline_with_matrix_data(LayerId::BASE, data);
    Some(app)
}

#[test]
fn large_grids_draw_thinned_meshes() {
    let Some(app) = built(1024) else {
        eprintln!("SKIPPED large_grids_draw_thinned_meshes: no GPU device");
        return;
    };
    let layer = &app.layers.base;
    let mesh = layer.data.mesh.as_ref().expect("thinned");
    // 1 M cells within at least 256 K: one in 2 along each axis.
    assert_eq!(mesh.stride, 2);
    let surface = layer.renderers.surface.as_ref().expect("surface");
    assert_eq!((surface.width, surface.height), (512, 512));
    let heatmap_cells = layer.data.matrix.as_ref().map(|m| m.values.len());
    assert_eq!(
        heatmap_cells,
        Some(1024 * 1024),
        "the heatmap keeps every cell"
    );
    // The middle cell of each 2 x 2.
    assert_eq!(mesh.matrix.values[0], 1024.0 + 1.0);
    let thinning = layer.mesh_thinning();
    assert_eq!(
        (thinning.stride, thinning.width, thinning.full_width),
        (2, 512, 1024)
    );
}

#[test]
fn small_grids_draw_every_cell() {
    let Some(app) = built(256) else {
        eprintln!("SKIPPED small_grids_draw_every_cell: no GPU device");
        return;
    };
    assert!(app.layers.base.data.mesh.is_none());
    assert_eq!(
        app.layers.base.mesh_thinning(),
        crate::plots::Thinning::NONE
    );
}
