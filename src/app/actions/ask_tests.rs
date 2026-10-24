//! Questions about the in-memory dataset and the plot.

use super::dims_tests::run;
use super::{AppAction, Done};
use crate::app::test_support::{TARGET, make_resident, memory_app};
use crate::app::{OctantApp, StoreKind};
use crate::data::{DatasetMetadata, SliceRequest, VariableInfo};

fn active_app() -> OctantApp {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    app.selected.store_target = TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    app
}

#[test]
fn listing_names_what_the_dataset_holds() {
    let mut app = active_app();
    let outcomes = app.run_utterance("list the variables");
    let [
        Ok(Done::DatasetShown {
            variables, entries, ..
        }),
    ] = outcomes.as_slice()
    else {
        panic!("{outcomes:?}");
    };
    assert_eq!(*variables, 4);
    assert!(entries.iter().any(|e| e == "t2m"), "{entries:?}");
}

#[test]
fn a_group_lists_its_own_variables() {
    let mut app = OctantApp::default();
    let var = |name: &str| VariableInfo {
        name: name.into(),
        ..Default::default()
    };
    let meta = DatasetMetadata {
        name: "earth".into(),
        variables: vec![var("/atmosphere/t2m"), var("/ocean/sst"), var("/ocean/sss")],
        ..Default::default()
    };
    app.load_new_metadata(meta);
    let outcome = app.dispatch(AppAction::DescribeDataset {
        group: Some("Ocean".into()),
    });
    let Ok(Done::DatasetShown {
        variables, entries, ..
    }) = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(
        (variables, entries),
        (2, vec!["sst".to_string(), "sss".to_string()])
    );
}

#[test]
fn a_search_fills_the_variables_panel_search() {
    let mut app = active_app();
    let outcomes = app.run_utterance("search sst");
    assert_eq!(
        outcomes,
        vec![Ok(Done::Searching {
            text: "sst".into(),
            matches: 1
        })]
    );
    assert_eq!(app.layout.variable_search, "sst");
}

#[test]
fn a_variable_is_described_with_its_dimensions() {
    let mut app = active_app();
    let outcomes = run(&mut app, "describe sst");
    let [Ok(Done::VariableShown { name, dims, .. })] = outcomes.as_slice() else {
        panic!("{outcomes:?}");
    };
    assert_eq!(name, "sst");
    let shown: Vec<(&str, usize)> = dims.iter().map(|d| (d.name.as_str(), d.size)).collect();
    assert_eq!(shown, [("time", 3), ("lat", 5), ("lon", 4)]);
}

#[test]
fn the_view_is_described_once_plotted() {
    let mut app = active_app();
    let before = app.run_utterance("what's plotted");
    assert!(
        matches!(before.as_slice(), [Ok(Done::DatasetShown { .. })]),
        "{before:?}"
    );
    app.run_utterance("plot t2m, then select sst");
    let after = app.run_utterance("what's plotted");
    let [
        Ok(Done::ViewShown {
            variable, staged, ..
        }),
    ] = after.as_slice()
    else {
        panic!("{after:?}");
    };
    assert_eq!((variable.as_str(), staged.as_deref()), ("t2m", Some("sst")));
}
