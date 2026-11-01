//! Detail tiles of a multi-channel composite the GPU blends: drawn from
//! their own channel planes, and kept when the channels' display changes,
//! which on the CPU re-blends and drops them.

use super::tests::{frame_at, rest_at, tile_level};
use crate::app::channel_gpu_tests::plotted;
use crate::plots::PlotType;

#[test]
fn display_changes_keep_a_gpu_blended_tile() {
    let Some(mut app) = plotted(true) else {
        eprintln!("SKIPPED display_changes_keep_a_gpu_blended_tile: no GPU device");
        return;
    };
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    assert!(
        app.layers.base.renderers.detail_channels.is_some(),
        "tile planes on the GPU"
    );
    let configs = &mut app.layers.base.composite.channel_configs;
    configs[1].color_rgb = [255, 255, 0];
    configs[2].visible = false;
    configs[0].window = Some((0.0, 10.0));
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    assert_eq!(tile_level(&app), Some(0), "the tile is kept");
    // The hover reads the tile in the new display.
    let tile = app.layers.base.data.detail.tile.as_ref().expect("tile");
    let [u0, v0, u1, v1] = tile.uv;
    let (x, y) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
    let reading = crate::ui::hover::detail::tile_reading(&app.layers.base, x, y).expect("reading");
    let probe = tile.probe.as_ref().expect("window");
    let stretch = app
        .layers
        .base
        .data
        .composite_stretch
        .as_ref()
        .expect("stretch");
    let configs = &app.layers.base.composite.channel_configs;
    let want = crate::data::slicing::blend_pixel(probe, configs, stretch, reading.cell);
    assert_eq!(reading.value.to_bits(), want.to_bits());
}

#[test]
fn display_changes_drop_a_cpu_blended_tile() {
    let Some(mut app) = plotted(false) else {
        return;
    };
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    assert!(app.layers.base.renderers.detail_channels.is_none());
    app.layers.base.composite.channel_configs[1].color_rgb = [255, 255, 0];
    frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    assert_eq!(tile_level(&app), None, "re-blended on the CPU");
}

#[test]
fn display_changes_drop_a_tile_too_big_for_the_gpu() {
    let Some(mut app) = plotted(true) else {
        eprintln!("SKIPPED display_changes_drop_a_tile_too_big_for_the_gpu: no GPU device");
        return;
    };
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    // As when the tile's planes did not fit: blended on the CPU.
    app.layers.base.renderers.detail_channels = None;
    app.layers.base.composite.channel_configs[1].color_rgb = [255, 255, 0];
    app.channel_display_changed(crate::app::layers::LayerId::BASE);
    assert_eq!(tile_level(&app), None, "the old colors are dropped");
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0), "asked for again");
}
