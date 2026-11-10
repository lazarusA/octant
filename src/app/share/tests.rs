//! Share links read into the actions they replay.

use super::parse_share;
use crate::app::actions::{AppAction, DimRole};

fn open(source: &str) -> AppAction {
    AppAction::OpenSource {
        source: source.into(),
    }
}

#[test]
fn a_link_reads_into_actions_in_a_fixed_order() {
    let link = "https://lazarusa.github.io/octant/?plot=heatmap&lat=30..60\
                &overlay=sst&range=250..310&src=https://x.org/era5.zarr&var=t2m\
                &time=2020-01-15T06:00&cmap=cmocean:thermal&axes=lon,lat&anim=time";
    let dim = |d: &str| Some(d.to_string());
    assert_eq!(
        parse_share(link),
        Some(vec![
            open("https://x.org/era5.zarr"),
            AppAction::SelectVariable { path: "t2m".into() },
            AppAction::SetRole {
                dim: "lon".into(),
                role: DimRole::X
            },
            AppAction::SetRole {
                dim: "lat".into(),
                role: DimRole::Y
            },
            AppAction::SetRole {
                dim: "time".into(),
                role: DimRole::Animate
            },
            AppAction::SetRange {
                dim: dim("lat"),
                from: "30".into(),
                to: "60".into()
            },
            AppAction::SetDim {
                dim: dim("time"),
                value: "2020-01-15T06:00".into()
            },
            AppAction::Plot {
                plot_type: Some("heatmap".into())
            },
            AppAction::SetColormap {
                name: "cmocean:thermal".into(),
                layer: None
            },
            AppAction::SetColorRange {
                min: 250.0,
                max: 310.0,
                layer: None
            },
            AppAction::SelectVariable { path: "sst".into() },
            AppAction::AddOverlay,
        ])
    );
}

#[test]
fn only_a_remote_src_makes_a_link() {
    assert_eq!(parse_share("https://x.org/a.zarr"), None);
    assert_eq!(parse_share("https://x.org/a.zarr?var=t2m"), None);
    assert_eq!(parse_share("?src=/home/me/a.zarr&var=t2m"), None);
    assert_eq!(parse_share("?src=file:///etc/passwd"), None);
    assert_eq!(
        parse_share("src=s3://bucket/a.zarr"),
        Some(vec![open("s3://bucket/a.zarr")])
    );
}

#[test]
fn escaped_values_and_presigned_sources_decode() {
    let link = "?src=https://x.org/a.zarr%3Fsig%3Da%26exp%3D1&var=air+temp&dim.plot=2&x=";
    assert_eq!(
        parse_share(link),
        Some(vec![
            open("https://x.org/a.zarr?sig=a&exp=1"),
            AppAction::SelectVariable {
                path: "air temp".into()
            },
            AppAction::SetDim {
                dim: Some("plot".into()),
                value: "2".into()
            },
            AppAction::Plot { plot_type: None },
        ])
    );
}

#[test]
fn a_repeated_key_takes_its_last_value() {
    let actions = parse_share("?src=https://x.org/a&var=a&var=b&t=1&t=2").unwrap_or_default();
    assert_eq!(actions[1], AppAction::SelectVariable { path: "b".into() });
    assert_eq!(
        actions[2],
        AppAction::SetDim {
            dim: Some("t".into()),
            value: "2".into()
        }
    );
}

#[test]
fn a_link_typed_as_a_source_opens_its_store() {
    let (mut app, _) = crate::app::test_support::memory_app();
    let ctx = egui::Context::default();
    app.submit_or_activate_source(
        "https://x.org/octant/?src=procedural://volume4d&plot=",
        None,
    );
    assert_ne!(app.selected.store_target, "procedural://volume4d", "queued");
    app.run_commands();
    assert_eq!(app.selected.store_target, "procedural://volume4d");
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.metadata_rx.is_some() {
        assert!(web_time::Instant::now() < deadline, "never settled");
        app.poll_inspection(&ctx);
        std::thread::yield_now();
    }
    assert!(app.selected.metadata.is_some(), "opened");
}

/// `tas(time, level, lat, lon)`, time in days since 2020-01-01, under a
/// remote address (registered, so nothing is fetched).
fn tas_app() -> crate::app::OctantApp {
    use crate::app::test_support::{write_array, write_coordinate};
    use serde_json::json;
    let store = std::sync::Arc::new(zarrs::storage::store::MemoryStore::new());
    let dims = ["time", "level", "lat", "lon"];
    write_array(&store, "tas", &dims, &[3, 2, 5, 4], &[1, 2, 5, 4]);
    let days = json!({ "units": "days since 2020-01-01" });
    write_coordinate(&store, "time", &[0.0, 31.0, 60.0], days);
    write_coordinate(&store, "level", &[500.0, 850.0], json!({ "units": "hPa" }));
    write_coordinate(&store, "lat", &[10.0, 20.0, 30.0, 40.0, 50.0], json!({}));
    write_coordinate(&store, "lon", &[0.0, 90.0, 180.0, 270.0], json!({}));
    app_of(store, "https://share.test/tas.zarr")
}

/// An app holding `store` as the remote dataset `target` (a URL of its own
/// per test: caches keyed by store URL are shared by tests running at once).
pub(super) fn app_of(
    store: std::sync::Arc<zarrs::storage::store::MemoryStore>,
    target: &str,
) -> crate::app::OctantApp {
    use crate::app::test_support::register_store;
    let mut app = crate::app::OctantApp::default();
    let meta = register_store(&mut app, store, target);
    let id = crate::app::StoreKind::make_source_id(crate::app::StoreKind::RemoteZarr, target);
    if let Some(dataset) = app.dataset_manager.get_mut(&id) {
        dataset.metadata = Some(meta);
    }
    app
}

/// Types `link` as a source and runs its commands until it plots and none
/// is left, none failing.
pub(super) fn apply(app: &mut crate::app::OctantApp, link: &str) {
    replay(app, link);
    assert!(app.toasts.is_empty(), "a command failed");
}

/// `apply`, whatever its commands' outcome.
pub(super) fn replay(app: &mut crate::app::OctantApp, link: &str) {
    let ctx = egui::Context::default();
    app.submit_or_activate_source(link, None);
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while app.plotted().metadata.is_none() || !app.commands_idle() {
        assert!(web_time::Instant::now() < deadline, "never plotted");
        app.poll_inspection(&ctx);
        app.poll_coordinate_results();
        app.poll_block_prefetch_results();
        app.run_commands();
        std::thread::yield_now();
    }
}

#[test]
fn a_plot_s_link_plots_the_same_again() {
    let cmap = crate::utils::colormap::registry::key_of(3);
    let mut first = tas_app();
    apply(
        &mut first,
        &format!(
            "?src=https://share.test/tas.zarr&var=tas&anim=time&time=2020-02-01\
             &level=850&lat=20..40&cmap={cmap}&range=250..310.5"
        ),
    );
    assert_eq!(first.playback.current_timestep, 1);
    let link = first.share_link().unwrap_or_default();
    for part in [
        "https://lazarusa.github.io/octant/?src=https://share.test/tas.zarr&var=tas",
        "axes=lon,lat",
        "anim=time",
        "step=2020-02-01",
        "level=850",
        "lat=20..40",
        "plot=heatmap",
        &format!("cmap={cmap}"),
        "range=250..310.5",
    ] {
        assert!(link.contains(part), "{part} missing from {link}");
    }
    let mut second = tas_app();
    apply(&mut second, &link);
    assert_eq!(second.share_link().unwrap_or_default(), link);
    assert_eq!(second.plotted().dim_config, first.plotted().dim_config);
    assert_eq!(second.playback.current_timestep, 1);
}

#[test]
fn a_local_store_has_no_link() {
    let (mut app, _) = crate::app::test_support::memory_app();
    app.selected.store_target = "/home/me/a.zarr".into();
    assert_eq!(app.share_link(), None);
}

#[test]
fn a_multiscale_plot_s_link_names_level_0() {
    use crate::app::test_support::{poll_until, pyramid_app, selection_of};
    let (mut app, meta) = pyramid_app();
    app.layout.canvas_px = [20, 20];
    app.selected = selection_of(&mut app, &meta, "0/t2m");
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.matrix.is_some());
    let sst = meta.variables.iter().position(|v| v.name == "0/sst");
    app.add_overlay(sst.unwrap_or_default());
    let shown = app.plotted().variable_info().map(|v| v.name.clone());
    assert_eq!(shown.as_deref(), Some("2/t2m"), "Auto reads level 2");
    app.layers.base.selection_mut().store_target = "https://x.org/p.zarr".into();
    let link = app.share_link().unwrap_or_default();
    assert!(link.contains("var=0/t2m&"), "{link}");
    assert!(link.contains("overlay=0/sst"), "{link}");
    assert!(
        link.contains("lat=%231..%2364"),
        "level 0's whole extent: {link}"
    );
}
