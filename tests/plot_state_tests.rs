//! The plotted variable's color state: a new variable resets the color range
//! to its data, a replot of the same variable keeps a locked range, and the
//! GPU color uniforms follow it. Drives the Plot button headlessly against the
//! offline procedural 4D store.

mod common;
use common::drain;

use octant::app::{OctantApp, StoreKind};
use octant::data::{BlockStore, backends::ProceduralBlockStore};
use octant::plots::PlotType;

fn plotted_heatmap() -> OctantApp {
    let mut app = OctantApp::default();
    let store = ProceduralBlockStore::open("procedural://volume4d").expect("open procedural store");
    let meta = store.inspect().expect("inspect procedural store");
    app.selected.store_kind = StoreKind::ProceduralVolume4D;
    app.selected.store_target = "procedural://volume4d".to_string();
    app.load_new_metadata(meta);
    app.show_hero = false;
    app.selected.plot_type = PlotType::Heatmap;
    app.plot_selection();
    drain(&mut app);
    app
}

fn select_variable(app: &mut OctantApp, idx: usize) {
    app.selected.variable_idx = idx;
    let info = app.selected.metadata.as_ref().expect("metadata").variables[idx].clone();
    octant::ui::variables_panel::init_variable_dimension_defaults(app, &info);
}

fn matrix_bounds(app: &OctantApp) -> (f32, f32) {
    let m = app.layers.base.data.matrix.as_ref().expect("matrix data");
    (m.min_val, m.max_val)
}

#[test]
fn a_new_variable_resets_a_locked_color_range() {
    let mut app = plotted_heatmap();
    assert_eq!(
        (
            app.layers.base.color.range_min,
            app.layers.base.color.range_max
        ),
        matrix_bounds(&app),
        "the first plot takes its data range"
    );
    app.layers.base.color.lock_bounds = true;
    app.layers.base.color.range_min = -123.0;
    app.layers.base.color.range_max = 456.0;

    let other = 1 - app.plotted().variable_idx;
    select_variable(&mut app, other);
    app.plot_selection();
    drain(&mut app);

    assert_eq!(
        app.plotted().variable_idx,
        other,
        "the new variable is plotted"
    );
    assert!(
        !app.layers.base.color.lock_bounds,
        "a new variable unlocks the range"
    );
    assert_eq!(
        (
            app.layers.base.color.range_min,
            app.layers.base.color.range_max
        ),
        matrix_bounds(&app),
        "a new variable takes its data range"
    );
}

#[test]
fn a_replot_of_the_same_variable_keeps_a_locked_range() {
    let mut app = plotted_heatmap();
    app.layers.base.color.lock_bounds = true;
    app.layers.base.color.range_min = -5.0;
    app.layers.base.color.range_max = 5.0;

    let anim = app.selected.animated_dim.expect("animated dimension");
    app.current_timestep = 7;
    app.selected.dim_indices[anim] = 7;
    app.plot_selection();
    drain(&mut app);

    assert!(app.layers.base.color.lock_bounds, "the range stays locked");
    assert_eq!(
        (
            app.layers.base.color.range_min,
            app.layers.base.color.range_max
        ),
        (-5.0, 5.0)
    );
    let params = app.get_color_params(&app.layers.base);
    assert_eq!((params.cmin, params.cmax), (-5.0, 5.0), "uniforms follow");
}

#[test]
fn reset_color_range_takes_the_plotted_data_bounds() {
    let mut app = plotted_heatmap();
    app.layers.base.color.lock_bounds = true;
    app.layers.base.color.range_min = -5.0;
    app.layers.base.color.range_max = 5.0;
    app.reset_color_range();
    assert!(!app.layers.base.color.lock_bounds);
    assert_eq!(
        (
            app.layers.base.color.range_min,
            app.layers.base.color.range_max
        ),
        matrix_bounds(&app)
    );
}
