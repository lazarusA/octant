//! Dates, times and labels said for a dimension, over `tas(time, lat)`.

use std::sync::Arc;

use serde_json::json;
use zarrs::storage::store::MemoryStore;

use super::dims_tests::{dim_index, run};
use super::{Done, Failure};
use crate::app::test_support::{TARGET, register_store, write_array, write_coordinate};
use crate::app::{OctantApp, StoreKind};

/// `tas(time, lat)` over `time` (in `units`) with `tas` staged.
fn timed_app(target: &str, time: &[f32], units: &str) -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    let n = time.len() as u64;
    write_array(&store, "tas", &["time", "lat"], &[n, 5], &[1, 5]);
    write_coordinate(&store, "time", time, json!({ "units": units }));
    open_tas(store, target)
}

/// `tas` of `store`, whose `lat` this writes, staged.
fn open_tas(store: Arc<MemoryStore>, target: &str) -> OctantApp {
    write_coordinate(&store, "lat", &[10.0, 20.0, 30.0, 40.0, 50.0], json!({}));
    let mut app = OctantApp::default();
    let meta = register_store(&mut app, store, target);
    app.selected.store_target = target.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    run(&mut app, "select tas");
    app
}

fn time_step(app: &OctantApp) -> usize {
    let time = dim_index(app, "time");
    if app.selected.animated_dim == Some(time) {
        app.playback.current_timestep
    } else {
        app.selected.dim_indices[time]
    }
}

#[test]
fn a_month_or_year_names_a_step_within_it() {
    // Monthly means stamped mid-month: Jan 15, Feb 15, Mar 16 at noon.
    let mut app = timed_app(
        "memory://monthly",
        &[14.5, 45.5, 75.5],
        "days since 2020-01-01",
    );
    run(&mut app, "time March 2020");
    assert_eq!(time_step(&app), 2);
    // Yearly stamps on July 2 at noon, as far from 2020-01-01 either way.
    let mut app = timed_app("memory://yearly", &[182.5, 548.5], "days since 2019-01-01");
    run(&mut app, "time 2020");
    assert_eq!(time_step(&app), 1);
    // A time of day, as a tool call writes it.
    let hourly: Vec<f32> = (0..24).map(|h| h as f32).collect();
    let mut app = timed_app("memory://hourly", &hourly, "hours since 2020-01-01");
    let set = app.dispatch(super::AppAction::SetDim {
        dim: Some("time".into()),
        value: "2020-01-01T12:00".into(),
    });
    assert!(set.is_ok(), "{set:?}");
    assert_eq!(time_step(&app), 12);
}

#[test]
fn a_dimension_s_name_outweighs_the_role_it_is_drawn_in() {
    // Time without CF attributes drawn on y, as a Hovmöller plot does.
    let mut app = timed_app("memory://hovmoller", &[0.0, 1.0, 2.0], "");
    run(&mut app, "time as y");
    let outcomes = run(&mut app, "latitude 30");
    assert!(
        matches!(outcomes.as_slice(), [Ok(Done::DimSet { dim, .. })] if dim == "lat"),
        "{outcomes:?}"
    );
}

#[test]
fn a_number_matches_text_labels() {
    use super::coord_match::{Dim, index_of};
    use crate::data::CoordValues;
    let (_, meta) = crate::app::test_support::memory_app();
    let var = &meta.variables[0];
    let dim = Dim {
        meta: &meta,
        var,
        store: TARGET,
        name: "station",
        len: 3,
    };
    let labels = CoordValues::Labels(["10", "11", "12"].map(String::from).to_vec().into());
    assert_eq!(index_of(&dim, Some(&labels), "11", false), Some(1));
}

#[test]
fn a_year_is_no_position_on_a_time_axis_without_coordinates() {
    let store = Arc::new(MemoryStore::new());
    write_array(&store, "tas", &["time", "lat"], &[2100, 5], &[100, 5]);
    let mut app = open_tas(store, "memory://uncoordinated");
    let outcomes = run(&mut app, "time 2020");
    assert!(
        matches!(outcomes.as_slice(), [Err(Failure::NoValue { .. })]),
        "{outcomes:?}"
    );
    assert!(matches!(
        run(&mut app, "time 5").as_slice(),
        [Ok(Done::DimSet { .. })]
    ));
    assert_eq!(time_step(&app), 4);
}
