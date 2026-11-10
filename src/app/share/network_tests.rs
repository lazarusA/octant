//! Links of catalog plots drawn as composites open the same plot: the
//! Sentinel-2 TCI GeoTIFF with its bands reordered, and IDR's nuclei
//! OME-Zarr image with one channel hidden. Run by hand with
//! `cargo test --lib share_links_of_composites -- --ignored --nocapture`.

use crate::app::OctantApp;
use crate::app::actions::{AppAction, Origin};

const TCI: &str = "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif";
const IDR: &str = "https://uk1s3.embassy.ebi.ac.uk/idr/zarr/v0.4/idr0062A/6001240.zarr";

/// Runs the app's loading and commands until `done`.
fn settle(app: &mut OctantApp, done: impl Fn(&OctantApp) -> bool) {
    let ctx = egui::Context::default();
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(120);
    while !done(app) {
        assert!(web_time::Instant::now() < deadline, "timed out");
        app.poll_inspection(&ctx);
        app.poll_coordinate_results();
        app.poll_block_prefetch_results();
        app.run_commands();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn plotted(app: &OctantApp) -> bool {
    app.layers.base.data.matrix.is_some()
        && app.metadata_rx.is_none()
        && !app.new_plot_pending()
        && app.commands_idle()
        && app.coordinate_loader.pending_count() == 0
}

/// Plots `var` of `store` as it opens, changed by `edit`, then opens its
/// link in another app and compares the two plots.
fn round_trip(store: &str, var: &str, edit: impl Fn(&mut OctantApp)) {
    let mut sender = OctantApp::default();
    let open = AppAction::OpenSource {
        source: store.into(),
    };
    let select = AppAction::SelectVariable { path: var.into() };
    let plot = AppAction::Plot { plot_type: None };
    sender.queue_actions(vec![open, select, plot], Origin::Link);
    settle(&mut sender, plotted);
    edit(&mut sender);
    let link = sender.share_link().unwrap_or_default();
    eprintln!("{link}");
    let mut receiver = OctantApp::default();
    receiver.submit_or_activate_source(&link, None);
    settle(&mut receiver, plotted);
    assert!(receiver.toasts.is_empty(), "a command failed");
    let (a, b) = (&sender.layers.base, &receiver.layers.base);
    assert_eq!(b.composite, a.composite);
    // Roles and ranges; an expanded dimension reads its range, not its index.
    let read = |app: &OctantApp| -> Vec<_> {
        let p = app.plotted();
        p.dim_config
            .iter()
            .map(|c| (c.spatial, c.animation, c.range))
            .collect()
    };
    assert_eq!(read(&receiver), read(&sender));
    assert_eq!(receiver.share_link().unwrap_or_default(), link);
}

#[test]
#[ignore = "network"]
fn share_links_of_composites() {
    round_trip(TCI, "raster", |app| {
        assert!(app.layers.base.composite.enabled, "TCI opens in color");
        app.layers.base.composite.rgb_channels = [2, 1, 0];
    });
    round_trip(IDR, "0", |app| {
        let configs = &mut app.layers.base.composite.channel_configs;
        assert_eq!(configs.len(), 2);
        configs[0].visible = false;
        configs[1].color_rgb = [255, 0, 255];
    });
}
