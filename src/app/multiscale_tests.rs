//! Plotting multiscale variables: the plot reads the level that fits the
//! canvas while the UI stays on level 0, and overlays follow that level.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, LevelChoice};
use crate::app::multiscale::{at_base, at_level, same_variable};
use crate::app::test_support::{poll_until, pyramid_app, selection_of};
use crate::data::DatasetMetadata;
use crate::plots::PlotType;

/// Stages `name` of the pyramid (level 0) on a canvas of `px` pixels.
fn stage(app: &mut OctantApp, meta: &DatasetMetadata, name: &str, px: u32) {
    app.layout.canvas_px = [px, px];
    app.selected = selection_of(app, meta, name);
}

fn plotted_name(app: &OctantApp) -> &str {
    app.plotted()
        .variable_info()
        .map_or("", |v| v.name.as_str())
}

#[test]
fn the_default_window_is_the_whole_extent() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    assert_eq!(app.selected.dim_ranges[1..], [(0, 63), (0, 63)]);
}

#[test]
fn auto_reads_the_finest_level_within_the_canvas() {
    let (mut app, meta) = pyramid_app();
    for (px, level) in [(20, 2), (40, 1), (64, 0), (500, 0), (8, 2)] {
        stage(&mut app, &meta, "0/t2m", px);
        assert_eq!(app.selection_level(&app.selected), Some(level), "{px} px");
    }
}

#[test]
fn a_plot_reads_its_level_while_the_ui_stays_on_level_0() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.dim_indices[0] = 2;
    app.selected.dim_ranges[0] = (0, 2);
    app.playback.current_timestep = 2;
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());

    assert_eq!(plotted_name(&app), "2/t2m");
    let shown = app.plotted();
    assert_eq!(shown.dim_ranges, [(0, 2), (0, 15), (0, 15)]);
    assert_eq!(shown.dim_indices[0], 2, "time indices are shared");
    let matrix = app.layers.base.data.matrix.as_ref().expect("matrix");
    assert_eq!((matrix.width, matrix.height), (16, 16));

    let staged = app.selected.variable_info().map(|v| v.name.as_str());
    assert_eq!(staged, Some("0/t2m"));
    assert_eq!(app.selected.dim_ranges[1..], [(0, 63), (0, 63)]);
    assert!(!app.is_exploring_unplotted_variable());
}

#[test]
fn a_pinned_level_is_read_as_chosen() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.level = LevelChoice::Pinned(1);
    assert_eq!(app.selection_level(&app.selected), Some(1));
    app.selected.level = LevelChoice::Pinned(9);
    assert_eq!(app.selection_level(&app.selected), Some(2));
}

#[test]
fn other_plot_types_read_level_0_when_it_fits() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.plot_type = PlotType::Surface;
    assert_eq!(app.selection_level(&app.selected), Some(0));
}

#[test]
fn levels_map_back_to_level_0() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.dim_ranges[1] = (8, 23);
    app.selected.dim_ranges[2] = (5, 5);
    app.selected.dim_indices[2] = 5;
    let level = at_level(&app.selected, 2).expect("level 2");
    assert_eq!(level.dim_ranges[1..], [(2, 5), (1, 1)]);
    assert_eq!(level.dim_indices[2], 1);
    let back = at_base(&level).expect("level 0");
    assert_eq!(back.variable_idx, app.selected.variable_idx);
    assert_eq!(back.dim_ranges[1..], [(8, 23), (6, 6)]);
    assert!(at_base(&app.selected).is_none(), "already level 0");
    let level2 = level.variable_idx;
    assert!(same_variable(&meta, level2, app.selected.variable_idx));
    let sst = meta.variables.iter().position(|v| v.name == "0/sst");
    assert!(!same_variable(&meta, level2, sst.expect("sst")));
}

#[test]
fn overlays_follow_the_base_level() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    let sst = meta.variables.iter().position(|v| v.name == "0/sst");
    let id = app.add_overlay(sst.expect("sst")).expect("overlay");
    let overlay = app.layers.get(id).expect("layer").selection();
    let name = overlay.variable_info().map(|v| v.name.as_str());
    assert_eq!(name, Some("2/sst"));
    assert_eq!(overlay.dim_ranges[1..], [(0, 15), (0, 15)]);

    // The same variable at another level is already a layer.
    let t2m = app.selected.variable_idx;
    assert!(app.overlay_unavailable_for(t2m).is_some());

    // A finer base plot takes the overlay along.
    stage(&mut app, &meta, "0/t2m", 40);
    app.plot_selection();
    poll_until(&mut app, |app| plotted_name(app) == "1/t2m");
    let overlay = app.layers.get(id).expect("layer").selection();
    let name = overlay.variable_info().map(|v| v.name.as_str());
    assert_eq!(name, Some("1/sst"));
    assert_ne!(id, LayerId::BASE);
}
