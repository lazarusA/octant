//! Surfaces and spheres of large 2D data draw a thinned grid, with a GPU.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::test_support::render_state;
use crate::data::MatrixData;

/// `n` x `n` cells built into the base layer's renderers on a 512 x 512
/// canvas, through a pyramid when `resampled`.
fn built_with(n: usize, resampled: bool) -> Option<OctantApp> {
    let mut app = OctantApp {
        wgpu_render_state: Some(render_state()?),
        enable_pyramid_resampling: resampled,
        ..Default::default()
    };
    app.layout.canvas_px = [512, 512];
    let values = (0..n * n).map(|i| i as f32).collect();
    let data = MatrixData::new(n, n, values, 0.0, (n * n) as f32, "m".into(), 1);
    app.rebuild_pipeline_with_matrix_data(LayerId::BASE, data);
    Some(app)
}

fn built(n: usize) -> Option<OctantApp> {
    built_with(n, false)
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

#[test]
fn a_resampled_heatmap_keeps_whole_meshes() {
    let Some(app) = built_with(1024, true) else {
        eprintln!("SKIPPED a_resampled_heatmap_keeps_whole_meshes: no GPU device");
        return;
    };
    let layer = &app.layers.base;
    assert!(
        layer.data.pyramid.is_some(),
        "the heatmap samples a pyramid"
    );
    // The surface thins the whole data, not the heatmap's sample of it.
    let mesh = layer.data.mesh.as_ref().expect("thinned");
    assert_eq!((mesh.matrix.width, mesh.stride), (512, 2));
    assert_eq!(mesh.matrix.values[0], 1024.0 + 1.0);
}

#[test]
fn the_detail_setting_thins_again() {
    let Some(mut app) = built(1024) else {
        eprintln!("SKIPPED the_detail_setting_thins_again: no GPU device");
        return;
    };
    let stride = |app: &OctantApp| app.layers.base.data.mesh.as_ref().map(|m| m.stride);
    let bake_id = |app: &OctantApp| {
        app.layers
            .base
            .renderers
            .surface
            .as_ref()
            .map(|s| s.bake_id)
    };
    let thinned = bake_id(&app);
    app.plot_configs.mesh.detail = crate::app::MeshDetail::Full;
    app.rethin_meshes();
    assert_eq!(stride(&app), None, "every cell");
    let surface = app.layers.base.renderers.surface.as_ref().map(|s| s.width);
    assert_eq!(surface, Some(1024));
    // A new mesh is a new bake target, so a GPU composite bakes into it.
    assert_ne!(bake_id(&app), thinned);
    app.plot_configs.mesh.detail = crate::app::MeshDetail::Screen;
    app.rethin_meshes();
    assert_eq!(stride(&app), Some(2), "thinned again");
    let kept = bake_id(&app);
    app.rethin_meshes();
    assert_eq!(bake_id(&app), kept, "the same stride keeps the mesh");
}

#[test]
fn a_larger_canvas_thins_less() {
    let Some(mut app) = built(1024) else {
        eprintln!("SKIPPED a_larger_canvas_thins_less: no GPU device");
        return;
    };
    let heatmap = app.layers.base.renderers.heatmap.clone();
    // 2048 x 1024 pixels show 1 M cells: every one.
    app.layout.canvas_px = [2048, 1024];
    app.rethin_meshes();
    assert!(app.layers.base.data.mesh.is_none());
    // Only the meshes are built again: resizing never rebuilds the heatmap.
    let kept = app.layers.base.renderers.heatmap.as_ref();
    assert!(
        heatmap
            .as_ref()
            .zip(kept)
            .is_some_and(|(a, b)| std::sync::Arc::ptr_eq(a, b))
    );
    assert_eq!(
        app.layers.base.renderers.surface.as_ref().map(|s| s.width),
        Some(1024)
    );
}
