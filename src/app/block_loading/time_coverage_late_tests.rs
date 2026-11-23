//! A selected variable whose time axis arrives later, over the slow
//! procedural store: its untouched slider moves into the stated span then.

use std::collections::HashMap;

use crate::app::OctantApp;
use crate::app::test_support::poll_until;
use crate::data::{CoordValues, VariableInfo};

/// The slow procedural store with an ARCO-ERA5-like span (days 4 to 9 of
/// 20, `days since 2000-01-01`), its time axis held back as a remote
/// store's arrives later.
fn slow_era5_app(delay: std::time::Duration) -> OctantApp {
    super::slow_store::unplotted_app(delay, None, |meta| {
        meta.dimension_coordinates
            .retain(|k, _| !k.ends_with("time"));
        let Some(var) = meta.variables.first_mut() else {
            return;
        };
        var.attributes
            .insert("valid_time_start".into(), "2000-01-05".into());
        var.attributes
            .insert("valid_time_stop".into(), "2000-01-10".into());
        meta.variables.push(VariableInfo {
            name: "time".into(),
            shape: vec![20],
            dimension_names: vec!["time".into()],
            units: Some("days since 2000-01-01".into()),
            ..VariableInfo::default()
        });
    })
}

/// The slow store's 20-day time axis arriving.
fn deliver_time_axis(app: &mut OctantApp) {
    let axis = CoordValues::Regular {
        start: 0.0,
        step: 1.0,
        len: 20,
    };
    let source_id = app.selected_source_id();
    app.merge_variable_coordinates(&source_id, HashMap::from([("time".to_string(), axis)]));
}

/// The time axis arrives after the variable was selected: its slider, left
/// alone, moves to the stated span's last step before any Plot.
#[test]
fn a_late_time_axis_moves_the_untouched_slider() {
    let mut app = slow_era5_app(std::time::Duration::from_millis(1));
    assert_eq!(app.selected.animated_dim(), Some(0));
    assert_eq!(
        app.playback.current_timestep, 0,
        "no time axis yet: no move"
    );
    deliver_time_axis(&mut app);
    // Day 9 (2000-01-10) is the stated span's last step.
    assert_eq!(app.playback.current_timestep, 9);
    let time = app.selected.dim_config[0];
    assert_eq!(time.index, 9);
    assert!((time.range.0..=time.range.1).contains(&9));
}

/// A slider the user moved before the time axis arrived stays where it is.
#[test]
fn a_slider_moved_before_the_time_axis_stays() {
    let mut app = slow_era5_app(std::time::Duration::from_millis(1));
    app.selected.hold_step(0, 3, 19);
    app.playback.current_timestep = 3;
    deliver_time_axis(&mut app);
    assert_eq!(app.playback.current_timestep, 3);
    assert_eq!(app.selected.dim_config[0].index, 3);
}

/// Plot pressed before the time axis arrived: what is plotted is never
/// moved afterwards.
#[test]
fn a_plot_before_the_time_axis_keeps_its_step() {
    let mut app = slow_era5_app(std::time::Duration::from_millis(1));
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    deliver_time_axis(&mut app);
    assert_eq!(app.playback.current_timestep, 0);
}

/// Opening the dataset selects its first variable and asks for its time
/// axis, so its slider moves before any click or Plot.
#[test]
fn opening_the_dataset_asks_for_the_first_variables_time_axis() {
    let app = slow_era5_app(std::time::Duration::from_millis(1));
    assert_eq!(app.coordinate_loader.pending_count(), 1);
}
