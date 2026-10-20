//! Playing a volume whose Z axis is also the animated one (time, lat, lon):
//! the steps inside one resident chunk only rotate the ring buffer, so they
//! neither slice the block again nor upload it.

use std::sync::Arc;

use zarrs::storage::store::MemoryStore;

use crate::app::test_support::{TARGET, poll_until, register_store, write_array};
use crate::app::{OctantApp, StoreKind};
use crate::plots::PlotType;

/// `pr(time, lat, lon)` (8x5x4, four steps per chunk) plotted as a volume.
fn volume_app() -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    write_array(
        &store,
        "pr",
        &["time", "lat", "lon"],
        &[8, 5, 4],
        &[4, 5, 4],
    );
    let mut app = OctantApp::default();
    let meta = register_store(&mut app, store, TARGET);
    let var = meta.variables[0].clone();
    app.selected.store_kind = StoreKind::RemoteZarr;
    app.selected.store_target = TARGET.to_string();
    app.load_new_metadata(meta);
    app.layout.show_hero = false;
    app.selected.variable_idx = 0;
    app.selected.plot_type = PlotType::Volume;
    crate::ui::variables_panel::init_variable_dimension_defaults(&mut app, &var);
    app.plot_selection();
    poll_until(&mut app, |a| a.layers.base.data.last_volume_block.is_some());
    app
}

#[test]
fn steps_within_a_resident_chunk_write_nothing() {
    let mut app = volume_app();
    assert_eq!(app.selected.animated_dim, Some(0), "time animates");
    app.playback.is_playing = true;
    let version = app.layers.base.data.volume_version;
    for step in 1..4 {
        app.advance_playback(web_time::Instant::now());
        assert_eq!(app.playback.current_timestep, step);
        assert_eq!(
            app.layers.base.data.volume_version, version,
            "step {step} wrote the same block again"
        );
    }

    // The next chunk is written once, when playback reaches it.
    poll_until(&mut app, |a| {
        a.layer_step_resident(crate::app::layers::LayerId::BASE, 4)
    });
    app.advance_playback(web_time::Instant::now());
    assert_eq!(app.playback.current_timestep, 4);
    assert_ne!(app.layers.base.data.volume_version, version);
}
