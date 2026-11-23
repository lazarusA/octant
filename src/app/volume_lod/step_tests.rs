//! Bricks of a `(time, depth, lat, lon)` pyramid read the shown time step,
//! indexed from their block's origin (`draw.rs` `block_axes_for`).

use super::encode::{Cube, brick_values};
use super::tests::rest;
use crate::app::layers::LevelChoice;
use crate::app::test_support::{poll_until, selection_of, time_depth_pyramid_app};
use crate::data::multiscale::bricks::BrickKey;
use crate::plots::PlotType;

#[test]
fn a_brick_holds_the_shown_time_step() {
    let (mut app, meta) = time_depth_pyramid_app();
    app.layout.canvas_px = [16, 16];
    let mut sel = selection_of(&mut app, &meta, "0/t2m");
    sel.plot_type = PlotType::Volume;
    sel.dim_config[1].range = (0, 15);
    sel.level = LevelChoice::Pinned(2);
    app.selected = sel;
    app.plot_selection();
    poll_until(&mut app, |a| a.layers.base.data.volume.is_some());
    app.layers.base.selection_mut().level = LevelChoice::Auto;
    app.plot_configs.volume.lod = true;
    app.playback.current_timestep = 1;
    rest(&mut app);

    let root = BrickKey {
        level: 2,
        b: [0; 3],
    };
    let read = app.brick_read(root).expect("the root's read");
    let block = app.block_cache.peek(&read.key).expect("the root's block");
    let lod = app
        .layers
        .base
        .data
        .volume_lod
        .lod
        .as_ref()
        .expect("bricked");
    let Some(Cube::Values(cube)) = brick_values(lod, root, &read, block) else {
        panic!("a scalar cube");
    };
    // Level 2 is 3 x 4 x 16 x 16 holding its flat index: step 1 is
    // 1024..2048 (f16 rounds by under one).
    let encoding = lod.encoding();
    let values: Vec<f32> = cube.iter().map(|&h| encoding.decode(h)).collect();
    assert!(
        values.iter().all(|v| (1023.0..=2048.0).contains(v)),
        "{:?}",
        values
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)))
    );
}
