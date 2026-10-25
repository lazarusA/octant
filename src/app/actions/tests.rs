//! Actions run against the in-memory app: names resolved from the dataset
//! as loaded, exploring kept apart from the plot, failures answered.

use super::resolve::{Resolution, Vocabulary};
use super::{AppAction, Done, Failure, NameKind};
use crate::app::test_support::{TARGET, make_resident, memory_app};
use crate::app::{OctantApp, StoreKind};
use crate::data::{DatasetMetadata, SliceRequest, VariableInfo};
use crate::plots::PlotType;

/// The memory dataset active, nothing plotted, every block resident.
fn active_app() -> (OctantApp, DatasetMetadata) {
    let (mut app, meta) = memory_app();
    for name in ["t2m", "sst"] {
        make_resident(&mut app, &SliceRequest::full_range(name, &[3, 5, 4]));
    }
    app.selected.store_target = TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta.clone());
    (app, meta)
}

fn index_of(meta: &DatasetMetadata, name: &str) -> usize {
    meta.variables
        .iter()
        .position(|v| v.name.trim_matches('/') == name)
        .expect("variable")
}

fn variable(name: &str, long_name: Option<&str>) -> VariableInfo {
    VariableInfo {
        name: name.to_string(),
        long_name: long_name.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn actions_round_trip_through_json() {
    let actions = [
        AppAction::SelectVariable {
            path: "atmosphere/t2m".into(),
        },
        AppAction::Plot {
            plot_type: Some("volume".into()),
        },
        AppAction::Plot { plot_type: None },
        AppAction::Play,
        AppAction::Pause,
        AppAction::Step { by: -2 },
        AppAction::SetColormap {
            name: "viridis".into(),
            layer: Some("sst".into()),
        },
        AppAction::AddOverlay,
        AppAction::SetLayerVisible {
            layer: "sst".into(),
            visible: false,
        },
        AppAction::RemoveOverlay {
            layer: "sst".into(),
        },
        AppAction::SetColorRange {
            min: 250.0,
            max: 300.0,
            layer: None,
        },
        AppAction::Export {
            format: Some("pdf".into()),
        },
        AppAction::OpenPlateWell {
            well: Some("B3".into()),
            field: Some(2),
        },
        AppAction::ResetView,
    ];
    for action in actions {
        let json = serde_json::to_string(&action).expect("serialize");
        let back: AppAction = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, action, "{json}");
    }
    let parsed: AppAction = serde_json::from_str(r#"{"action":"plot"}"#).expect("plot");
    assert_eq!(parsed, AppAction::Plot { plot_type: None });
}

#[test]
fn variables_resolve_by_path_suffix_name_and_long_name() {
    let meta = DatasetMetadata {
        variables: vec![
            variable("/atmosphere/t2m", None),
            variable("/forecast/t2m", None),
            variable("/ocean/sst", Some("Sea Surface Temperature")),
        ],
        ..Default::default()
    };
    let vocab = Vocabulary::new(Some(&meta), &[]);
    assert_eq!(vocab.variable("forecast t2m"), Resolution::Resolved(1));
    assert_eq!(
        vocab.variable("T 2 M"),
        Resolution::Ambiguous(vec!["atmosphere/t2m".into(), "forecast/t2m".into()])
    );
    assert_eq!(
        vocab.variable("sea surface temperature"),
        Resolution::Resolved(2)
    );
    assert_eq!(vocab.variable("SST"), Resolution::Resolved(2));
    let Resolution::NotFound(closest) = vocab.variable("sstt") else {
        panic!("sstt is no variable");
    };
    assert_eq!(closest.first().map(String::as_str), Some("ocean/sst"));
}

#[test]
fn plot_types_and_colormaps_resolve_in_any_language() {
    let vocab = Vocabulary::new(None, &[]);
    assert_eq!(
        vocab.plot_type("volumen"),
        Resolution::Resolved(PlotType::Volume)
    );
    assert_eq!(
        vocab.plot_type("ヒートマップ"),
        Resolution::Resolved(PlotType::Heatmap)
    );
    assert_eq!(
        vocab.plot_type("point cloud"),
        Resolution::Resolved(PlotType::PointCloud)
    );
    let Resolution::Resolved(id) = vocab.colormap("Viridis") else {
        panic!("viridis is a colormap");
    };
    assert!(crate::utils::colormap::registry::key_of(id).ends_with("viridis"));
}

#[test]
fn a_spoken_plot_selects_and_plots() {
    let (mut app, meta) = active_app();
    let outcomes = app.run_utterance("Plot T2M.");
    assert_eq!(
        outcomes,
        vec![
            Ok(Done::Selected("t2m".into())),
            Ok(Done::Plotted {
                variable: "t2m".into(),
                plot_type: PlotType::Heatmap
            }),
        ]
    );
    assert_eq!(app.plotted().variable_idx, index_of(&meta, "t2m"));
    assert!(app.layers.base.data.matrix.is_some(), "t2m is drawn");
}

#[test]
fn exploring_leaves_the_plot_while_playback_and_colors_reach_it() {
    let (mut app, meta) = active_app();
    app.run_utterance("plot t2m");
    let outcomes = app.run_utterance("select sst, then play, colormap viridis");
    assert!(outcomes.iter().all(Result::is_ok), "{outcomes:?}");
    assert_eq!(
        app.selected.variable_idx,
        index_of(&meta, "sst"),
        "sst is staged"
    );
    assert_eq!(
        app.plotted().variable_idx,
        index_of(&meta, "t2m"),
        "t2m stays plotted"
    );
    assert!(app.playback.is_playing);
    let key = crate::utils::colormap::registry::key_of(app.layers.base.color.colormap);
    assert!(key.ends_with("viridis"), "{key}");
}

#[test]
fn a_mistaken_name_answers_with_the_closest_and_stops() {
    let (mut app, _) = active_app();
    let staged = app.selected.variable_idx;
    let outcomes = app.run_utterance("plot sstt then play");
    let [
        Err(Failure::NotFound {
            kind: NameKind::Variable,
            closest,
            ..
        }),
    ] = outcomes.as_slice()
    else {
        panic!("{outcomes:?}");
    };
    assert_eq!(closest.first().map(String::as_str), Some("sst"));
    assert_eq!(app.selected.variable_idx, staged, "nothing else staged");
    assert!(!app.playback.is_playing, "the rest did not run");
}

#[test]
fn playback_needs_a_plot_with_an_animated_dimension() {
    let (mut app, _) = active_app();
    assert_eq!(
        app.run_utterance("play"),
        vec![Err(Failure::NothingToAnimate)]
    );
    let (mut empty, _) = memory_app();
    assert_eq!(
        empty.run_utterance("plot t2m"),
        vec![Err(Failure::NoDataset)]
    );
}

#[test]
fn unknown_text_is_not_understood() {
    let (mut app, _) = active_app();
    let outcomes = app.run_utterance("good morning");
    assert_eq!(
        outcomes,
        vec![Err(Failure::NotUnderstood("good morning".into()))]
    );
}

#[test]
fn text_not_understood_is_said_in_the_bar_not_as_a_toast() {
    let failure = Failure::NotUnderstood("good morning".into());
    assert!(super::answer::failure_notice(&failure).is_none());
    assert!(super::answer::not_understood_line("good morning").ends_with("good morning"));
}

#[test]
fn steps_move_by_the_count_asked_even_before_they_load() {
    let (mut app, meta) = memory_app();
    make_resident(&mut app, &SliceRequest::full_range("sst", &[1, 5, 4]));
    app.selected.store_target = TARGET.to_string();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.load_new_metadata(meta);
    app.run_utterance("plot sst");
    let stepped = app.dispatch(AppAction::Step { by: 2 });
    assert_eq!(stepped, Ok(Done::Stepped(2)));
    crate::app::test_support::poll_until(&mut app, |app| app.playback.current_timestep == 2);
    let far = AppAction::Step { by: -2_000_000_000 };
    app.dispatch(far)
        .expect("one step around the series, however far");
}

#[test]
fn every_family_s_key_names_its_colormap() {
    use crate::utils::colormap::registry;
    let vocab = Vocabulary::new(None, &[]);
    for id in 0..registry::len() as u32 {
        let key = registry::key_of(id);
        assert_eq!(vocab.colormap(&key), Resolution::Resolved(id), "{key}");
    }
}
