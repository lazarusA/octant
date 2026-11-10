//! Plotting multiscale variables: the plot reads the level that fits the
//! canvas while the UI stays on level 0, and overlays follow that level.

use crate::app::OctantApp;
use crate::app::layers::{LayerId, LevelChoice};
use crate::app::multiscale::{at_base, at_level, same_variable};
use crate::app::test_support::{
    big_pyramid_app, iso_pyramid_app, poll_until, pyramid_app, ranges, selection_of,
};
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
    assert_eq!(ranges(&app.selected)[1..], [(0, 63), (0, 63)]);
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
    app.selected.dim_config[0].index = 2;
    app.selected.dim_config[0].range = (0, 2);
    app.playback.current_timestep = 2;
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());

    assert_eq!(plotted_name(&app), "2/t2m");
    let shown = app.plotted();
    assert_eq!(ranges(shown), [(0, 2), (0, 15), (0, 15)]);
    assert_eq!(shown.dim_config[0].index, 2, "time indices are shared");
    let matrix = app.layers.base.data.matrix.as_ref().expect("matrix");
    assert_eq!((matrix.width, matrix.height), (16, 16));

    let staged = app.selected.variable_info().map(|v| v.name.as_str());
    assert_eq!(staged, Some("0/t2m"));
    assert_eq!(ranges(&app.selected)[1..], [(0, 63), (0, 63)]);
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
fn three_d_plot_types_read_the_finest_level_within_a_buffer() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 500);
    for plot_type in [
        PlotType::Surface,
        PlotType::Sphere,
        PlotType::Volume,
        PlotType::PointCloud,
    ] {
        app.selected.plot_type = plot_type;
        app.selected.level = LevelChoice::Auto;
        assert_eq!(app.selection_level(&app.selected), Some(0), "{plot_type:?}");
        app.selected.level = LevelChoice::Pinned(2);
        assert_eq!(app.selection_level(&app.selected), Some(2), "{plot_type:?}");
    }
}

#[test]
fn sizes_and_gpu_limits_are_measured_on_the_level_read() {
    use crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS;
    use crate::ui::variables_panel::{calculate_selected_volume_elements, is_volume_allowed};
    let (mut app, meta) = big_pyramid_app();
    stage(&mut app, &meta, "0/t2m", 500);
    app.selected.plot_type = PlotType::Volume;
    assert_eq!(app.selection_level(&app.selected), Some(2));
    let (read, dims) = app.read_window(&app.selected).expect("window");
    assert_eq!(read.name.trim_matches('/'), "2/t2m");
    assert_eq!((dims[1].range, dims[2].range), ((0, 2047), (0, 2047)));
    assert_eq!(ranges(&app.selected)[1..], [(0, 8191), (0, 8191)]);
    assert!(calculate_selected_volume_elements(&app) <= MAX_GPU_STORAGE_BUFFER_ELEMENTS);
    assert!(is_volume_allowed(&app, &app.selected, None));
    let options = crate::ui::plot_type::options::plot_options(&app);
    assert!(options.iter().all(|o| o.blocked.is_none()), "{options:?}");
    // Level 0 is past the limit: pinned, a coarser level within it is read.
    app.selected.level = LevelChoice::Pinned(0);
    assert_eq!(app.selection_level(&app.selected), Some(1));
    assert!(is_volume_allowed(&app, &app.selected, None));
}

#[test]
fn line_plots_read_the_level_a_volume_reads() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.plot_type = PlotType::Line;
    let volume = app.selection_level_as(&app.selected, PlotType::Volume);
    assert_eq!(volume, Some(0), "small: the finest");
    assert_eq!(app.selection_level(&app.selected), volume);
}

#[test]
fn levels_map_back_to_level_0() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.dim_config[1].range = (8, 23);
    app.selected.dim_config[2].range = (5, 5);
    app.selected.dim_config[2].index = 5;
    let level = at_level(&app.selected, 2).expect("level 2");
    assert_eq!(ranges(&level)[1..], [(2, 5), (1, 1)]);
    assert_eq!(level.dim_config[2].index, 1);
    let back = at_base(&level).expect("level 0");
    assert_eq!(back.variable_idx, app.selected.variable_idx);
    assert_eq!(
        ranges(&back)[1..],
        [(8, 23), (5, 5)],
        "the index it came from"
    );
    assert_eq!(back.dim_config[2].index, 5);
    let mut moved = level.clone();
    (moved.dim_config[2].range, moved.dim_config[2].index) = ((2, 2), 2);
    let back = at_base(&moved).expect("level 0");
    assert_eq!(
        back.dim_config[2].range,
        (10, 10),
        "a new index: the cell's center"
    );
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
    assert_eq!(ranges(overlay)[1..], [(0, 15), (0, 15)]);

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

#[test]
fn a_block_arriving_after_a_resize_plots_its_own_level() {
    let (mut app, meta) = pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.plot_selection();
    // The canvas grows while level 2's block is fetched: Auto would now
    // read level 0, but the block that arrives is level 2's.
    app.layout.canvas_px = [500, 500];
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    assert_eq!(plotted_name(&app), "2/t2m");
}

#[test]
fn a_collapsed_downsampled_dimension_reads_coarse_levels() {
    let (mut app, meta) = iso_pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    assert_eq!(
        app.selected.dim_config[0].range.0,
        app.selected.dim_config[0].range.1
    );
    assert_eq!(app.selection_level(&app.selected), Some(2));
    // Animated, depth steps are level-0 indices: only level 0 keeps them.
    app.selected.dim_config[0].animation = crate::app::AnimationRole::Animated;
    assert_eq!(app.selection_level(&app.selected), Some(0));
}

#[test]
fn a_plotted_slice_keeps_its_level_0_index() {
    let (mut app, meta) = iso_pyramid_app();
    stage(&mut app, &meta, "0/t2m", 20);
    app.selected.dim_config[0].index = 13;
    app.selected.dim_config[0].range = (13, 13);
    app.selected.dim_config[0].index = 13;
    app.selected.dim_config[0].range = (13, 13);
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());

    assert_eq!(plotted_name(&app), "2/t2m");
    assert_eq!(app.plotted().dim_config[0].range, (3, 3), "13 / 4");
    app.copy_plotted_to_selected();
    assert_eq!(app.selected.dim_config[0].index, 13);
    assert_eq!(app.selected.dim_config[0].range, (13, 13));
    assert_eq!(app.selected.dim_config[0].index, 13);
    let finer = crate::app::multiscale::to_level(app.plotted(), 1).expect("level 1");
    assert_eq!(
        finer.dim_config[0].range,
        (6, 6),
        "13 / 2, not the coarse cell's center"
    );
}
