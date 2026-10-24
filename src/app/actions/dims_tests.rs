//! Dimension actions over a dataset with coordinates: `tas(time, level,
//! lat, lon)`, its time in days since 2020-01-01, levels 500 and 850 hPa.

use std::sync::Arc;

use serde_json::json;
use zarrs::storage::store::MemoryStore;

use super::{DimRole, Done, Failure};
use crate::app::test_support::{TARGET, register_store, write_array, write_coordinate};
use crate::app::{OctantApp, SpatialRole, StoreKind};

/// The store's own address: coordinates are cached by address.
const DIMS_TARGET: &str = "memory://dims";

fn dims_app() -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    let dims = ["time", "level", "lat", "lon"];
    write_array(&store, "tas", &dims, &[3, 2, 5, 4], &[1, 2, 5, 4]);
    let days = json!({ "units": "days since 2020-01-01" });
    write_coordinate(&store, "time", &[0.0, 31.0, 60.0], days);
    write_coordinate(&store, "level", &[500.0, 850.0], json!({ "units": "hPa" }));
    write_coordinate(&store, "lat", &[10.0, 20.0, 30.0, 40.0, 50.0], json!({}));
    write_coordinate(&store, "lon", &[0.0, 90.0, 180.0, 270.0], json!({}));
    let mut app = OctantApp::default();
    let meta = register_store(&mut app, store, DIMS_TARGET);
    app.selected.store_target = DIMS_TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    app
}

/// Runs `utterance`, letting the coordinates it waits for arrive.
pub(super) fn run(app: &mut OctantApp, utterance: &str) -> Vec<super::Outcome> {
    let mut outcomes = app.run_utterance(utterance);
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.coordinate_loader.pending_count() > 0 {
        assert!(
            web_time::Instant::now() < deadline,
            "coordinates never came"
        );
        app.poll_coordinate_results();
        std::thread::yield_now();
    }
    app.poll_coordinate_results();
    outcomes.extend(app.resume_commands());
    outcomes
}

pub(super) fn dim_index(app: &OctantApp, name: &str) -> usize {
    let var = app.selected_variable_info().expect("staged");
    var.dimension_names
        .iter()
        .position(|d| d == name)
        .expect("dim")
}

#[test]
fn a_spoken_date_moves_the_time_step_once_the_coordinates_are_read() {
    let mut app = dims_app();
    let outcomes = run(&mut app, "select tas, then go to February 2020");
    let [Ok(Done::Selected(_)), Ok(Done::DimSet { dim, .. })] = outcomes.as_slice() else {
        panic!("{outcomes:?}");
    };
    assert_eq!(dim, "time");
    let time = dim_index(&app, "time");
    let shown = if app.selected.animated_dim == Some(time) {
        app.playback.current_timestep
    } else {
        app.selected.dim_indices[time]
    };
    assert_eq!(shown, 1, "February is the second step");
}

#[test]
fn a_number_picks_the_nearest_coordinate_and_two_a_range() {
    let mut app = dims_app();
    run(&mut app, "select tas");
    let level = run(&mut app, "level 840");
    assert!(
        matches!(level.as_slice(), [Ok(Done::DimSet { .. })]),
        "{level:?}"
    );
    assert_eq!(app.selected.dim_indices[dim_index(&app, "level")], 1);
    let lat = run(&mut app, "latitude 20 to 40");
    assert!(
        matches!(lat.as_slice(), [Ok(Done::RangeSet { .. })]),
        "{lat:?}"
    );
    assert_eq!(app.selected.dim_ranges[dim_index(&app, "lat")], (1, 3));
}

#[test]
fn roles_change_as_the_dimensions_panel_changes_them() {
    let mut app = dims_app();
    run(&mut app, "select tas");
    let outcomes = run(&mut app, "level as z");
    assert_eq!(
        outcomes,
        vec![Ok(Done::RoleSet {
            dim: "level".into(),
            role: DimRole::Z
        })]
    );
    assert_eq!(
        app.selected.dim_config[dim_index(&app, "level")].spatial,
        SpatialRole::Z
    );
}

#[test]
fn a_value_beyond_the_dimension_is_answered() {
    let mut app = dims_app();
    run(&mut app, "select tas");
    let outcomes = run(&mut app, "time 1990");
    assert!(
        matches!(outcomes.as_slice(), [Ok(Done::DimSet { .. })]),
        "nearest: {outcomes:?}"
    );
    // Without coordinates a number is a 1-based position.
    let (mut plain, meta) = crate::app::test_support::memory_app();
    plain.selected.store_target = TARGET.to_string();
    plain.selected.store_kind = StoreKind::RemoteZarr;
    plain.load_new_metadata(meta);
    run(&mut plain, "select t2m");
    assert!(matches!(
        run(&mut plain, "time 2").as_slice(),
        [Ok(Done::DimSet { .. })]
    ));
    let outcomes = run(&mut plain, "time 9");
    assert!(
        matches!(outcomes.as_slice(), [Err(Failure::NoValue { .. })]),
        "{outcomes:?}"
    );
}

/// `ta(record, pres, j, i)`: names no heuristic reads; only the
/// coordinates' CF attributes say what each dimension is.
fn cf_named_app() -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    let dims = ["record", "pres", "j", "i"];
    write_array(&store, "ta", &dims, &[3, 2, 5, 4], &[1, 2, 5, 4]);
    let days = json!({ "units": "days since 2020-01-01" });
    write_coordinate(&store, "record", &[0.0, 31.0, 60.0], days);
    write_coordinate(&store, "pres", &[500.0, 850.0], json!({ "units": "hPa" }));
    let lat = json!({ "standard_name": "latitude" });
    write_coordinate(&store, "j", &[10.0, 20.0, 30.0, 40.0, 50.0], lat);
    write_coordinate(
        &store,
        "i",
        &[0.0, 90.0, 180.0, 270.0],
        json!({ "axis": "X" }),
    );
    let mut app = OctantApp::default();
    let meta = register_store(&mut app, store, "memory://cf-named");
    app.selected.store_target = "memory://cf-named".to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    app
}

#[test]
fn axis_words_find_dimensions_by_their_cf_attributes() {
    let mut app = dims_app_like(cf_named_app());
    let said = [
        ("time 2020-02-01", "record"),
        ("level 850", "pres"),
        ("latitude 40", "j"),
        ("longitude 90", "i"),
    ];
    for (utterance, expected) in said {
        let outcomes = run(&mut app, utterance);
        let [Ok(Done::DimSet { dim, .. })] = outcomes.as_slice() else {
            panic!("'{utterance}': {outcomes:?}");
        };
        assert_eq!(dim, expected, "'{utterance}'");
    }
    // Said in Spanish, by the language's own axis word.
    let es = crate::i18n::with_language(crate::i18n::Language::Spanish, || {
        run(&mut app, "latitud 20")
    });
    assert!(
        matches!(es.as_slice(), [Ok(Done::DimSet { dim, .. })] if dim == "j"),
        "{es:?}"
    );
}

/// `app` with its variable `ta` staged.
fn dims_app_like(mut app: OctantApp) -> OctantApp {
    run(&mut app, "select ta");
    app
}

#[test]
fn cf_attributes_alone_say_what_a_dimension_is() {
    use super::axes::{Axis, axis_of};
    use crate::data::{DatasetMetadata, VariableInfo};
    let coord = |name: &str, key: &str, value: &str| VariableInfo {
        name: name.into(),
        attributes: [(key.to_string(), value.to_string())].into(),
        ..Default::default()
    };
    let ta = VariableInfo {
        name: "ta".into(),
        dimension_names: ["record", "pres", "j", "i", "band"]
            .map(String::from)
            .to_vec(),
        ..Default::default()
    };
    let meta = DatasetMetadata {
        variables: vec![
            ta.clone(),
            coord("record", "units", "hours since 1979-01-01"),
            coord("pres", "positive", "down"),
            coord("j", "standard_name", "grid_latitude"),
            coord("i", "axis", "X"),
        ],
        ..Default::default()
    };
    let axes: Vec<Option<Axis>> = (0..5).map(|i| axis_of(&meta, &ta, i, None)).collect();
    assert_eq!(
        axes,
        [
            Some(Axis::Time),
            Some(Axis::Level),
            Some(Axis::Latitude),
            Some(Axis::Longitude),
            None
        ]
    );
}
