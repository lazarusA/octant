//! Bricks over the in-memory isotropic pyramid (depth, lat, lon 16 x 64 x
//! 64, halved per level; one brick per level, values their flat index)
//! plotted as a volume at level 2, as a stack too large for level 0 lands:
//! the root arrives at once, a close-up brings in level 0 and the hover
//! reads its value (MIP the largest inside the voxel); a far view or
//! "Detail by view" off read nothing.

use crate::app::OctantApp;
use crate::app::layers::LevelChoice;
use crate::app::pipeline::paint_layer::CanvasView;
use crate::app::test_support::{iso_pyramid_app, poll_until, selection_of};
use crate::data::multiscale::bricks::BrickKey;
use crate::plots::PlotType;

pub(super) fn plotted() -> OctantApp {
    let (mut app, meta) = iso_pyramid_app();
    app.layout.canvas_px = [16, 16];
    let mut sel = selection_of(&mut app, &meta, "0/t2m");
    sel.plot_type = PlotType::Volume;
    sel.dim_config[0].range = (0, 15);
    sel.level = LevelChoice::Pinned(2);
    app.selected = sel;
    app.plot_selection();
    poll_until(&mut app, |a| a.layers.base.data.volume.is_some());
    app.layers.base.selection_mut().level = LevelChoice::Auto;
    app.plot_configs.volume.lod = true;
    app
}

fn view(px: f32) -> CanvasView {
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(px, px));
    CanvasView {
        canvas_rect: rect,
        plot_rect: rect,
        pan: [0.0, 0.0],
        zoom: 1.0,
        aspect_scale: [1.0, 1.0],
    }
}

thread_local! {
    static CLOCK: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

/// Frames a second apart until the view rested and every brick asked for
/// arrived.
pub(super) fn rest(app: &mut OctantApp) {
    let px = app.layout.canvas_px[1] as f32;
    for frame in 0..20 {
        let now = CLOCK.with(|c| {
            c.set(c.get() + 1.0);
            c.get()
        });
        app.refresh_volume_lod(PlotType::Volume, &view(px), now);
        let idle = |a: &OctantApp| {
            a.layers
                .base
                .data
                .volume_lod
                .lod
                .as_ref()
                .is_none_or(|l| l.pending.is_empty())
        };
        poll_until(app, idle);
        let done = !app.layers.base.data.volume_lod.busy();
        // Three frames a second apart settle any view.
        if done && frame >= 3 {
            return;
        }
    }
}

fn holds(app: &OctantApp, level: u8) -> bool {
    let key = BrickKey { level, b: [0; 3] };
    app.layers
        .base
        .data
        .volume_lod
        .lod
        .as_ref()
        .is_some_and(|l| l.residency.holds(&key))
}

/// Just in front of the corner at +x, +y of the front face, on a large
/// canvas: level 0's voxels span many pixels.
fn close_up(app: &mut OctantApp) {
    app.layout.canvas_px = [1000, 1000];
    app.nav.sphere_rotation_x = 0.0;
    app.nav.sphere_rotation_y = 0.0;
    app.nav.sphere_zoom = 0.02;
    app.nav.orbit_target = [0.45, 0.45, 0.13];
}

#[test]
fn the_root_arrives_and_a_far_view_wants_no_more() {
    let mut app = plotted();
    rest(&mut app);
    assert!(holds(&app, 2), "the root (level 2)");
    assert!(
        app.layers
            .base
            .data
            .volume_lod
            .lod
            .as_ref()
            .is_some_and(|l| l.ready())
    );
    assert!(!holds(&app, 1) && !holds(&app, 0), "16 px fit level 2");
    let (level, _) = app.brick_reading([0, 0, 0], [16, 16, 4]).expect("bricked");
    assert_eq!(level, 2);
}

#[test]
fn bricks_on_their_way_keep_the_view_unsettled() {
    let mut app = plotted();
    app.refresh_volume_lod(PlotType::Volume, &view(16.0), 0.5);
    assert!(!app.view_settled(), "the root is asked for at once");
    rest(&mut app);
    assert!(app.view_settled());
}

#[test]
fn a_close_up_reads_level_0_and_the_hover_its_value() {
    let mut app = plotted();
    rest(&mut app);
    close_up(&mut app);
    rest(&mut app);
    assert!(holds(&app, 0), "level 0 brought in");
    // Latitude is drawn north up, opposite to storage.
    assert_eq!(app.layers.base.data.flipped_dims, ["lat"]);
    // Plotted voxel (15, 0, 0) of 16 x 16 x 4: the top row covers stored
    // level-0 rows 60..64, so its center is x 62, y 62, z 2, whose value is
    // its flat index 2 * 4096 + 62 * 64 + 62.
    let (level, value) = app.brick_reading([15, 0, 0], [16, 16, 4]).expect("bricked");
    assert_eq!((level, value), (0, 12222.0));
    // MIP draws the largest level-0 value inside the voxel (x, y 60..64, z
    // 0..4): x 63, y 63, z 3.
    app.plot_configs.volume.algorithm = 1;
    let (level, value) = app.brick_reading([15, 0, 0], [16, 16, 4]).expect("bricked");
    assert_eq!((level, value), (0, 3.0 * 4096.0 + 63.0 * 64.0 + 63.0));
}

#[test]
fn off_reads_nothing() {
    let mut app = plotted();
    app.plot_configs.volume.lod = false;
    rest(&mut app);
    assert!(app.layers.base.data.volume_lod.lod.is_none());

    app.plot_configs.volume.lod = true;
    rest(&mut app);
    assert!(holds(&app, 2));
    app.plot_configs.volume.lod = false;
    rest(&mut app);
    assert!(
        app.layers.base.data.volume_lod.lod.is_none(),
        "turned off drops the bricks"
    );
    assert!(app.brick_reading([0, 0, 0], [16, 16, 4]).is_none());
}

#[test]
fn an_abort_while_loading_asks_again() {
    let mut app = plotted();
    let now = 1e6;
    app.refresh_volume_lod(PlotType::Volume, &view(16.0), now);
    app.abort_current_fetch();
    rest(&mut app);
    assert!(holds(&app, 2), "the root asked again after the abort");
}

#[test]
fn a_single_value_range_keeps_finer_values_finite() {
    use super::placement::value_encoding;
    assert_eq!(value_encoding(0.0, 0.0), (0.0, 1.0));
    assert_eq!(value_encoding(500.0, 500.0), (500.0, 500.0));
    assert_eq!(value_encoding(2.0, 10.0), (2.0, 8.0));
    assert_eq!(value_encoding(f32::NAN, 1.0), (0.0, 1.0));
    // A finer value past f16's range saturates rather than reading as none.
    let far = super::encode::encode_value(1e9, 0.0, 1.0);
    assert_eq!(half::f16::from_bits(far), half::f16::MAX);
}

#[test]
fn integer_ids_too_far_apart_for_f16_stay_dense() {
    use super::placement::{F16_WHOLE_SPAN, wide_integers};
    let ids = [0.0, 1.0, 5000.0, f32::NAN];
    assert!(wide_integers("uint16", &ids, (0.0, 5000.0)));
    assert!(wide_integers("<u4", &ids, (0.0, 5000.0)));
    assert!(
        !wide_integers("uint16", &ids, (0.0, F16_WHOLE_SPAN)),
        "f16 holds them"
    );
    assert!(!wide_integers("float32", &ids, (0.0, 5000.0)), "not ids");
    assert!(!wide_integers("u8", &ids, (0.0, 5000.0)), "a byte here");
    assert!(
        !wide_integers("int32", &[0.5, 5000.0], (0.0, 5000.0)),
        "packed reals"
    );
}

/// Two refreshes a second apart without polling: the view settles and its
/// reads start, none arrives.
fn settle_without_arrivals(app: &mut OctantApp) {
    let px = app.layout.canvas_px[1] as f32;
    for _ in 0..2 {
        let now = CLOCK.with(|c| {
            c.set(c.get() + 1.0);
            c.get()
        });
        app.refresh_volume_lod(PlotType::Volume, &view(px), now);
    }
}

#[test]
fn reads_the_view_moved_away_from_are_cancelled() {
    let mut app = plotted();
    rest(&mut app);
    close_up(&mut app);
    settle_without_arrivals(&mut app);
    let pending = |a: &OctantApp| {
        let lod = a.layers.base.data.volume_lod.lod.as_ref().expect("bricked");
        lod.pending.keys().cloned().collect::<Vec<_>>()
    };
    let asked = pending(&app);
    assert!(!asked.is_empty(), "the close-up reads finer bricks");

    // Back to the far view, which wants only the root.
    app.layout.canvas_px = [16, 16];
    app.nav.sphere_zoom = 1.0;
    app.nav.orbit_target = [0.0; 3];
    settle_without_arrivals(&mut app);
    assert!(pending(&app).is_empty(), "nothing the view wants is read");
    for key in &asked {
        assert!(!app.block_prefetcher.is_pending(key), "{key:?} still asked");
    }
}
