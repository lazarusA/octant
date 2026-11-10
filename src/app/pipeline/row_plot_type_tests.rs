//! A block of one row plots as a line, with a GPU.

use std::sync::Arc;

use zarrs::storage::store::MemoryStore;

use crate::app::test_support::{TARGET, poll_until, register_store, render_state, write_array};
use crate::app::{OctantApp, StoreKind};
use crate::plots::PlotType;

/// Plots `row` (1 x 8) as a heatmap; with `browse`, stages the 2D `grid`
/// before its block arrives. `None` without a GPU.
fn plotted_row(browse: bool) -> Option<OctantApp> {
    let store = Arc::new(MemoryStore::new());
    write_array(&store, "row", &["lat", "lon"], &[1, 8], &[1, 8]);
    write_array(&store, "grid", &["lat", "lon"], &[5, 4], &[5, 4]);
    let mut app = OctantApp {
        wgpu_render_state: Some(render_state()?),
        ..Default::default()
    };
    let meta = register_store(&mut app, store, TARGET);
    let index = |name| meta.variables.iter().position(|v| v.name == name);
    let (row, grid) = (index("row")?, index("grid")?);
    let var = meta.variables.get(row)?.clone();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.selected.store_target = TARGET.to_string();
    app.load_new_metadata(meta);
    app.layout.show_hero = false;
    app.selected.variable_idx = row;
    app.selected.plot_type = PlotType::Heatmap;
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    app.plot_selection();
    if browse {
        app.selected.variable_idx = grid;
    }
    poll_until(&mut app, |a| a.layers.base.data.matrix.is_some());
    Some(app)
}

#[test]
fn a_row_shows_as_a_line() {
    let Some(app) = plotted_row(false) else {
        eprintln!("SKIPPED a_row_shows_as_a_line: no GPU device");
        return;
    };
    assert_eq!(app.effective_canvas_plot_type(), PlotType::Line);
    assert_eq!(app.selected.plot_type, PlotType::Line, "the picker agrees");
}

#[test]
fn a_row_leaves_the_browsed_variable_s_plot_type() {
    let Some(app) = plotted_row(true) else {
        eprintln!("SKIPPED a_row_leaves_the_browsed_variable_s_plot_type: no GPU device");
        return;
    };
    assert_eq!(app.effective_canvas_plot_type(), PlotType::Line);
    assert_eq!(
        app.selected.plot_type,
        PlotType::Heatmap,
        "browsing keeps it"
    );
}
