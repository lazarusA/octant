//! What thinning a surface again costs when the canvas size changes its
//! stride (`OctantApp::rethin_meshes`), timed by hand:
//! `cargo test --release --lib rethin_timed -- --ignored --nocapture`.
//! A grid that fits one storage buffer, and one drawn through a pyramid.

use std::time::Instant;

use super::channel_blend_bench_tests::{RUNS, median};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::test_support::render_state;
use crate::data::MatrixData;
use crate::plots::PlotType;

/// 26 M cells (one buffer) and 50 M cells (past it: a pyramid).
const SIZES: [(usize, usize); 2] = [(7200, 3600), (10_000, 5000)];

/// Canvases whose mesh budgets give different strides.
const CANVASES: [[u32; 2]; 2] = [[1920, 1080], [960, 540]];

#[test]
#[ignore = "benchmark"]
fn rethin_timed() {
    let Some(rs) = render_state() else {
        eprintln!("SKIPPED rethin_timed: no GPU device");
        return;
    };
    eprintln!("surface thinned again (median of {RUNS} runs)");
    for (w, h) in SIZES {
        let mut app = OctantApp {
            wgpu_render_state: Some(rs.clone()),
            ..Default::default()
        };
        app.selected.plot_type = PlotType::Surface;
        app.plot_configs.mesh.detail = crate::app::MeshDetail::Screen;
        app.layout.canvas_px = CANVASES[0];
        let values: Vec<f32> = (0..w * h).map(|i| (i % 1000) as f32).collect();
        let matrix = MatrixData::new(w, h, values, 0.0, 999.0, "m".into(), 1);
        app.rebuild_pipeline_with_matrix_data(LayerId::BASE, matrix);
        let times = (0..RUNS)
            .map(|run| {
                app.layout.canvas_px = CANVASES[(run + 1) % 2];
                let start = Instant::now();
                app.rethin_meshes();
                let _ = rs.device.poll(wgpu::PollType::wait_indefinitely());
                start.elapsed()
            })
            .collect();
        let stride = app.layers.base.data.mesh.as_ref().map_or(1, |m| m.stride);
        let cells = (w * h) as f64 / 1e6;
        eprintln!(
            "{w:>6} x {h:<5} {cells:>5.1} M cells, stride {stride}: {:>10.2?}",
            median(times)
        );
    }
}
