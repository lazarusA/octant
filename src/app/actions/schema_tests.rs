//! The tools: one per action, named names listed, calls made actions.

use serde_json::json;

use super::schema::{Names, action_of, tools};
use super::{AppAction, Done, Failure};
use crate::app::StoreKind;
use crate::app::test_support::{TARGET, memory_app};

fn tool<'t>(all: &'t [super::schema::Tool], name: &str) -> &'t super::schema::Tool {
    all.iter().find(|t| t.name == name).expect("tool")
}

#[test]
fn every_action_is_a_tool_with_a_description() {
    let all = tools(&Names::default());
    let names: Vec<&str> = all.iter().map(|t| t.name.as_str()).collect();
    for expected in [
        "select_variable",
        "plot",
        "set_dim",
        "set_role",
        "open_plate_well",
        "export",
        "describe_view",
    ] {
        assert!(names.contains(&expected), "{expected} in {names:?}");
    }
    assert_eq!(all.len(), 22, "one per AppAction variant: {names:?}");
    for t in &all {
        assert!(!t.description.is_empty(), "{}", t.name);
        assert_eq!(t.input_schema["type"], "object", "{}", t.name);
        assert!(
            t.input_schema["properties"].get("action").is_none(),
            "{}",
            t.name
        );
    }
    // Shared definitions are inlined: each tool's input stands alone.
    let role = &tool(&all, "set_role").input_schema["properties"]["role"];
    assert!(role.get("$ref").is_none(), "{role}");
}

#[test]
fn a_dataset_s_names_are_listed_while_few_enough() {
    let names = Names {
        variables: vec!["t2m".into(), "sst".into()],
        dimensions: vec!["time".into()],
        ..Names::default()
    };
    let all = tools(&names);
    let path = &tool(&all, "select_variable").input_schema["properties"]["path"];
    assert_eq!(path["enum"], json!(["t2m", "sst"]));
    // An optional argument may still be null.
    let dim = &tool(&all, "set_dim").input_schema["properties"]["dim"];
    assert_eq!(dim["enum"], json!(["time", null]));
    let many = Names {
        variables: (0..500).map(|i| format!("v{i}")).collect(),
        ..Names::default()
    };
    let all = tools(&many);
    assert!(
        tool(&all, "select_variable").input_schema["properties"]["path"]
            .get("enum")
            .is_none()
    );
}

#[test]
fn calls_become_actions_or_say_why_not() {
    assert_eq!(
        action_of("set_dim", &json!({ "value": "2020-01-15" })),
        Ok(AppAction::SetDim {
            dim: None,
            value: "2020-01-15".into(),
        })
    );
    assert_eq!(
        action_of("reset_view", &json!(null)),
        Ok(AppAction::ResetView)
    );
    assert!(
        action_of("set_dim", &json!({})).is_err(),
        "value is required"
    );
    assert!(action_of("launch_rockets", &json!({})).is_err());
    let (mut app, meta) = memory_app();
    app.selected.store_target = TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    assert_eq!(
        app.call_tool("select_variable", &json!({ "path": "sst" })),
        Ok(Done::Selected("sst".into()))
    );
    assert!(matches!(
        app.call_tool("plot", &json!({ "plot_type": 3 })),
        Err(Failure::NotUnderstood(_))
    ));
    let names = app.tools();
    let path = &tool(&names, "select_variable").input_schema["properties"]["path"];
    assert!(
        path["enum"]
            .as_array()
            .is_some_and(|v| v.contains(&json!("t2m"))),
        "{path}"
    );
}
