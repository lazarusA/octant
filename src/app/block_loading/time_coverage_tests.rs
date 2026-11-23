//! A new plot of an ARCO-ERA5-like variable starts inside the span its
//! attributes state; selecting it, or a plotted variable's own steps, never
//! move the shared step.

use std::collections::HashMap;

use crate::app::OctantApp;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

/// Hourly from 1900, as ARCO-ERA5 allocates it; data from 1940 to 2026-11-17.
const STEPS: u64 = 1_323_648;
const LAST_VALID: usize = 1_112_207; // 2026-11-17T23 in hours since 1900

fn era5_app() -> OctantApp {
    let mut attributes = HashMap::new();
    attributes.insert("valid_time_start".to_string(), "1940-01-01".to_string());
    attributes.insert(
        "valid_time_stop_era5t".to_string(),
        "2026-11-17".to_string(),
    );
    let t2m = VariableInfo {
        name: "t2m".into(),
        data_type: "float32".into(),
        shape: vec![STEPS, 4, 8],
        dimension_names: vec!["time".into(), "latitude".into(), "longitude".into()],
        chunk_shape: vec![1, 4, 8],
        attributes,
        ..VariableInfo::default()
    };
    let time = VariableInfo {
        name: "time".into(),
        data_type: "int64".into(),
        shape: vec![STEPS],
        dimension_names: vec!["time".into()],
        units: Some("hours since 1900-01-01 00:00:00".into()),
        ..VariableInfo::default()
    };
    let mut coords = HashMap::new();
    let axis = CoordValues::Regular {
        start: 0.0,
        step: 1.0,
        len: STEPS as usize,
    };
    coords.insert("time".to_string(), axis);
    let mut app = OctantApp::default();
    app.selected.metadata = Some(DatasetMetadata {
        name: "era5".into(),
        store_type: "Zarr".into(),
        variables: vec![t2m.clone(), time],
        dimension_coordinates: coords,
        ome: None,
    });
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &t2m);
    app
}

#[test]
fn selecting_a_variable_shows_the_last_step_with_data() {
    // The slider shows it before Plot: the staged step, its window and the
    // shared step the animated slider reads.
    let app = era5_app();
    assert_eq!(app.playback.current_timestep, LAST_VALID);
    let time = app.selected.dim_config[0];
    assert_eq!(time.index, LAST_VALID);
    assert!((time.range.0..=time.range.1).contains(&LAST_VALID));
}

/// Selecting another variable on the same time axis: its defaults put the
/// window at the first chunk while the shared step stays on the data; the
/// window moves there too, so the slider and Plot show the data.
#[test]
fn selecting_another_variable_moves_its_window_to_the_data() {
    let mut app = era5_app();
    let Some(var) = app.selected.variable_info().cloned() else {
        panic!("a selected variable");
    };
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    let time = app.selected.dim_config[0];
    assert!(
        (time.range.0..=time.range.1).contains(&LAST_VALID),
        "{time:?}"
    );
    app.plot_selection();
    assert_eq!(app.playback.current_timestep, LAST_VALID);
}

/// Switching back to another open dataset: its defaults are applied while
/// the ERA5 metadata is still selected, which must not move it by ERA5's span.
#[test]
fn switching_datasets_ignores_the_previous_datasets_span() {
    let mut app = era5_app();
    app.selected.hold_step(0, 0, STEPS as usize - 1); // 1900, outside ERA5's data
    app.playback.current_timestep = 0;
    let sst = VariableInfo {
        name: "sst".into(),
        data_type: "float32".into(),
        shape: vec![100, 4, 8],
        dimension_names: vec!["time".into(), "latitude".into(), "longitude".into()],
        chunk_shape: vec![1, 4, 8],
        ..VariableInfo::default()
    };
    app.load_new_metadata(DatasetMetadata {
        name: "other".into(),
        store_type: "Zarr".into(),
        variables: vec![sst],
        dimension_coordinates: HashMap::new(),
        ome: None,
    });
    assert_eq!(app.playback.current_timestep, 0);
    let time = app.selected.dim_config[0];
    assert!(time.index < 100 && time.range.1 < 100, "{time:?}");
}

#[test]
fn plot_shows_the_step_the_slider_shows() {
    let mut app = era5_app();
    // 1900, outside the data, picked on purpose.
    app.selected.hold_step(0, 5, STEPS as usize - 1);
    app.playback.current_timestep = 5;
    app.plot_selection();
    assert_eq!(app.playback.current_timestep, 5);
    assert_eq!(app.selected.dim_config[0].index, 5);
}

#[test]
fn a_constant_block_holds_no_data() {
    use super::arrival_notices::holds_no_data;
    assert!(holds_no_data(&[f32::NAN, f32::NAN]));
    // A Zarr `fill_value` of 0 that no attribute masks.
    assert!(holds_no_data(&[0.0, 0.0, f32::NAN]));
    assert!(!holds_no_data(&[0.0, 1.0]));
}

#[test]
fn an_empty_step_outside_the_data_warns_once_per_run() {
    let mut app = era5_app();
    *app.layers.base.selection_mut() = app.selected.clone();
    let block = |values: Vec<f32>| {
        let shape = vec![values.len()];
        crate::data::OctantBlock::new(
            "t2m".into(),
            shape,
            vec!["longitude".into()],
            vec![0],
            values,
            HashMap::new(),
            HashMap::new(),
        )
        .expect("a one-dimensional block")
    };
    let (empty, full) = (block(vec![f32::NAN; 4]), block(vec![1.0, 2.0]));
    let warned = |app: &OctantApp| app.toasts.items().iter().map(|t| t.count).sum::<u32>();
    app.playback.current_timestep = 0;
    app.notice_empty_block(&empty);
    assert_eq!(warned(&app), 1);
    // Further empty steps outside the data, however far, stay quiet, and a
    // step outside with values does not end the run.
    app.playback.current_timestep = 24 * 365;
    app.notice_empty_block(&empty);
    app.notice_empty_block(&full);
    app.playback.current_timestep = 24 * 366;
    app.notice_empty_block(&empty);
    assert_eq!(warned(&app), 1);
    // A step with data ends the run; the next empty step outside warns again.
    app.playback.current_timestep = LAST_VALID;
    app.notice_empty_block(&full);
    app.playback.current_timestep = 2;
    app.notice_empty_block(&empty);
    assert_eq!(warned(&app), 2);
    // A step inside shown from the cache (staged, no arrival) ends it too.
    app.playback.current_timestep = LAST_VALID;
    app.stage_current_step();
    app.playback.current_timestep = 900;
    app.notice_empty_block(&empty);
    assert_eq!(warned(&app), 3);
}

#[test]
fn plot_keeps_a_range_chosen_inside_the_data() {
    let mut app = era5_app();
    // 1950s, inside the data; the shared step is still 1900's.
    let (start, end) = (438_288, 525_960);
    app.selected.set_range(0, (start, end), start);
    app.playback.current_timestep = 0;
    app.plot_selection();
    assert_eq!(app.playback.current_timestep, start);
    assert_eq!(app.selected.dim_config[0].range, (start, end));
}

#[test]
fn plot_keeps_the_selected_step() {
    let mut app = era5_app();
    app.plot_selection();
    assert_eq!(app.playback.current_timestep, LAST_VALID);
}

/// Only the shown step's slab of a block holding several steps counts.
#[test]
fn only_the_shown_steps_slab_decides_an_empty_step() {
    let mut app = era5_app();
    *app.layers.base.selection_mut() = app.selected.clone();
    // Steps 0 and 1 of `time`, two cells each: step 0 empty, step 1 filled.
    let values = vec![f32::NAN, f32::NAN, 1.0, 2.0];
    let block = crate::data::OctantBlock::new(
        "t2m".into(),
        vec![2, 2],
        vec!["time".into(), "longitude".into()],
        vec![0, 0],
        values,
        HashMap::new(),
        HashMap::new(),
    )
    .expect("a two-step block");
    let warned = |app: &OctantApp| app.toasts.items().len();
    app.playback.current_timestep = 1;
    app.notice_empty_block(&block);
    assert_eq!(warned(&app), 0);
    app.playback.current_timestep = 0;
    app.notice_empty_block(&block);
    assert_eq!(warned(&app), 1);
}
