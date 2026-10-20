//! Playing a volume whose Z axis is also the animated one (time, lat, lon):
//! the volume is a ring of the window of steps from the current one, and each
//! step writes only the plane the window gained, never a whole block.

use std::sync::Arc;

use zarrs::storage::store::MemoryStore;

use crate::app::layers::LayerId;
use crate::app::test_support::{TARGET, poll_until, register_store, write_array};
use crate::app::{OctantApp, StoreKind};
use crate::plots::PlotType;

const STEPS: usize = 8;
/// Steps the volume shows: the selected time range `0..=3`.
const DEPTH: usize = 4;

/// `pr(time, lat, lon)` (8x5x4, four steps per chunk) plotted as a volume of
/// four steps.
fn volume_app() -> OctantApp {
    let store = Arc::new(MemoryStore::new());
    write_array(
        &store,
        "pr",
        &["time", "lat", "lon"],
        &[STEPS as u64, 5, 4],
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
    app.selected.dim_ranges[0] = (0, DEPTH - 1);
    app.plot_selection();
    poll_until(&mut app, |a| {
        a.layers.base.data.volume.is_some() && !a.ring_missing(LayerId::BASE)
    });
    app
}

/// The step plane `slot` holds, by its smallest value (`write_array` fills
/// step `t` with `20t..20t + 20`), `None` when empty.
fn held_step(app: &OctantApp, slot: usize) -> Option<usize> {
    let volume = app.layers.base.data.volume.as_ref()?;
    let plane = volume.values.get(volume.plane_range(slot, 1))?;
    let min = plane
        .iter()
        .copied()
        .filter(|v| !v.is_nan())
        .reduce(f32::min)?;
    Some(min as usize / 20)
}

/// Asserts that the ring shows steps `start..start + DEPTH`, empty past the end.
fn assert_window(app: &OctantApp, start: usize) {
    for t in start..start + DEPTH {
        let expected = (t < STEPS).then_some(t);
        assert_eq!(
            held_step(app, t % DEPTH),
            expected,
            "step {start}: plane of {t}"
        );
    }
}

#[test]
fn each_step_writes_only_the_plane_the_window_gains() {
    let mut app = volume_app();
    assert_eq!(app.selected.animated_dim, Some(0), "time animates");
    assert!(app.animates_along_z(LayerId::BASE), "time is the Z axis");
    assert_eq!(
        app.layers.base.data.volume.as_ref().map(|v| v.depth),
        Some(DEPTH)
    );
    assert_window(&app, 0);

    app.playback.is_playing = true;
    for step in 1..STEPS {
        let back = (step + DEPTH - 1).min(STEPS - 1);
        poll_until(&mut app, |a| a.layer_step_resident(LayerId::BASE, back));
        app.layers.base.renderers.volume_dirty = None;
        app.advance_playback(web_time::Instant::now());
        assert_eq!(app.playback.current_timestep, step);
        assert_window(&app, step);
        let dirty = app.layers.base.renderers.volume_dirty.clone();
        assert_eq!(
            dirty.map(|z| z.len()),
            Some(1),
            "step {step} uploads the one plane it gained"
        );
    }
}

#[test]
fn playback_waits_for_the_window_s_last_plane() {
    let mut app = volume_app();
    assert_eq!(app.ring_lead(LayerId::BASE), DEPTH - 1);
    // Step 1 needs step 4, the second chunk's first, which is resident once
    // the first image settled; drop it.
    app.block_cache.clear();
    app.playback.is_playing = true;
    app.playback.buffer_secs = 0.0;
    assert!(!app.may_show(1), "step 4 is not loaded");
}
