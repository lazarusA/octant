//! A plot takes the selection its block was requested from, and the step
//! buttons step the plotted view while another variable is staged.

use crate::app::OctantApp;
use crate::app::test_support::{make_resident, memory_app, poll_until, ranges, selection_of};
use crate::data::SliceRequest;

fn name(selection: &crate::app::layers::VariableSelection) -> Option<String> {
    selection
        .variable_info()
        .map(|v| v.name.trim_matches('/').to_string())
}

fn plotted_name(app: &OctantApp) -> Option<String> {
    name(app.plotted())
}

#[test]
fn an_arriving_block_plots_the_variable_it_was_requested_for() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "elev");
    app.plot_selection();
    // Another variable is staged before elev's block arrives.
    app.selected = selection_of(&mut app, &meta, "rgb");
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    assert_eq!(plotted_name(&app).as_deref(), Some("elev"));
    assert_eq!(
        name(&app.selected).as_deref(),
        Some("rgb"),
        "rgb stays staged"
    );
}

#[test]
fn step_buttons_step_the_plot_while_another_variable_is_staged() {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.plot_selection();
    assert_eq!(plotted_name(&app).as_deref(), Some("t2m"));
    app.selected = selection_of(&mut app, &meta, "elev");
    app.step_next();
    assert_eq!(app.playback.current_timestep, 1);
    assert_eq!(plotted_name(&app).as_deref(), Some("t2m"), "the plot steps");
    let asked = app.layers.base.load.slice_request.as_ref();
    let asked = asked.map(|r| r.variable.trim_matches('/'));
    assert_eq!(asked, Some("t2m"), "no block of the staged elev is asked");
    assert_eq!(
        name(&app.selected).as_deref(),
        Some("elev"),
        "elev stays staged"
    );
}

/// `t2m` plotted at step 0 from a fetched block (step 1 is not resident).
fn t2m_fetched() -> (OctantApp, crate::data::DatasetMetadata) {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    (app, meta)
}

#[test]
fn a_step_fetched_while_another_variable_is_staged_is_shown() {
    let (mut app, meta) = t2m_fetched();
    app.selected = selection_of(&mut app, &meta, "elev");
    app.step_next();
    poll_until(&mut app, |app| app.playback.current_timestep == 1);
    assert_eq!(plotted_name(&app).as_deref(), Some("t2m"));
    assert_eq!(name(&app.selected).as_deref(), Some("elev"));
}

#[test]
fn a_selection_left_by_a_failed_fetch_is_never_plotted() {
    let (mut app, meta) = t2m_fetched();
    let mut stale = app.selected.clone();
    stale.variable_idx = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == "sst")
        .expect("sst");
    app.layers.base.load.requested = Some(stale);
    app.step_next();
    poll_until(&mut app, |app| app.playback.current_timestep == 1);
    assert_eq!(plotted_name(&app).as_deref(), Some("t2m"));
}

#[test]
fn a_step_pressed_while_a_new_plot_loads_keeps_the_new_plot() {
    let (mut app, meta) = t2m_fetched();
    app.selected = selection_of(&mut app, &meta, "sst");
    app.plot_selection();
    app.step_next();
    poll_until(&mut app, |app| plotted_name(app).as_deref() == Some("sst"));
    assert!(app.layers.base.data.matrix.is_some());
}

#[test]
fn a_multiscale_plot_steps_on_its_level_while_another_variable_is_staged() {
    let (mut app, meta) = crate::app::test_support::pyramid_app();
    app.layout.canvas_px = [20, 20];
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    let before = app.plotted().clone();
    assert_eq!(plotted_name(&app).as_deref(), Some("2/t2m"));
    app.selected = selection_of(&mut app, &meta, "0/sst");
    app.step_next();
    poll_until(&mut app, |app| app.playback.current_timestep == 1);
    assert_eq!(plotted_name(&app).as_deref(), Some("2/t2m"));
    assert_eq!(ranges(app.plotted())[1..], ranges(&before)[1..]);
    assert_eq!(name(&app.selected).as_deref(), Some("0/sst"));
}

#[test]
fn a_step_pressed_while_a_new_range_loads_keeps_the_new_range() {
    let (mut app, _) = t2m_fetched();
    app.selected.dim_config[1].range = (1, 3);
    app.plot_selection();
    app.step_next();
    poll_until(&mut app, |app| app.plotted().dim_config[1].range == (1, 3));
}

#[test]
fn the_staged_time_follows_the_step_with_the_dimensions_panel_closed() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    // Time collapsed to one index, as its slider's Range toggle leaves it.
    app.selected.dim_config[0].active = false;
    app.selected.set_index(0, 0);
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());

    app.step_next();
    poll_until(&mut app, |app| app.playback.current_timestep == 1);
    let time = app.selected.dim_config[0];
    assert_eq!((time.index, time.range), (1, (1, 1)));
    assert_eq!(app.plotted().dim_config[0], time, "the plot shows the same");
    // Plotting again keeps the step it shows.
    app.plot_selection();
    assert_eq!(app.playback.current_timestep, 1);
}
