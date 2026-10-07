//! Slider coordinate labels: one value for an index, both ends for a range, none without
//! coordinates.

use std::collections::HashMap;

use super::coord_label::format_label;
use crate::app::OctantApp;
use crate::data::{CoordValues, DatasetMetadata, VariableInfo};

fn app_with(coords: Option<(&str, CoordValues)>) -> (OctantApp, VariableInfo) {
    let var = VariableInfo {
        name: "temp".into(),
        shape: vec![6, 3],
        dimension_names: vec!["depth".into(), "region".into()],
        ..Default::default()
    };
    let dimension_coordinates = coords
        .map(|(dim, values)| HashMap::from([(dim.to_string(), values)]))
        .unwrap_or_default();
    let meta = DatasetMetadata {
        variables: vec![var.clone()],
        dimension_coordinates,
        ..Default::default()
    };
    let app = OctantApp {
        active_dataset_metadata: Some(meta),
        ..Default::default()
    };
    (app, var)
}

#[test]
fn an_index_shows_its_coordinate_and_a_range_both_ends() {
    let levels = vec![0.0, 10.0, 25.0, 50.0, 100.0, 200.0];
    let depth = CoordValues::from_values(levels, false).expect("depth");
    let (app, var) = app_with(Some(("depth", depth)));
    assert_eq!(
        format_label(&app, &var, 0, (3, 3)).as_deref(),
        Some("50.00 m")
    );
    assert_eq!(
        format_label(&app, &var, 0, (1, 4)).as_deref(),
        Some("10.00 m - 100.00 m")
    );
}

#[test]
fn labels_read_as_stored() {
    let names = ["Europe", "Africa", "Asia"].map(String::from).to_vec();
    let regions = CoordValues::from_labels(names).expect("regions");
    let (app, var) = app_with(Some(("region", regions)));
    assert_eq!(format_label(&app, &var, 1, (2, 2)).as_deref(), Some("Asia"));
}

#[test]
fn dimensions_without_coordinates_show_nothing() {
    let (app, var) = app_with(None);
    assert_eq!(format_label(&app, &var, 0, (2, 2)), None);
    assert_eq!(
        format_label(&app, &var, 5, (0, 0)),
        None,
        "unknown dimension"
    );
}
