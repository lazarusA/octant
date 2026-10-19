//! Detail tiles of an RGB composite: the in-memory pyramid's three time
//! steps drawn as red, green and blue, read finer when zoomed in, with the
//! plot's stretch and a window for the hover.

use super::tests::{plotted_with, rest_at, tile_level};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::app::state::AnimationRole;
use crate::app::test_support::poll_until;
use crate::plots::PlotType;

/// `t2m`'s three steps plotted as an RGB composite at level 2, turned on
/// as the Settings checkbox does (a new variable takes its defaults).
fn composite() -> OctantApp {
    let mut app = plotted_with(|_, sel| {
        sel.dim_config[0].animation = AnimationRole::None;
        sel.dim_config[0].range = (0, 2);
        sel.dim_ranges[0] = (0, 2);
        sel.animated_dim = None;
    });
    app.layers.base.composite.enabled = true;
    app.load_layer_block(LayerId::BASE);
    poll_until(&mut app, |app| {
        app.layers.base.data.composite_probe.is_some()
    });
    app
}

#[test]
fn composites_read_finer_levels_when_zoomed_in() {
    let mut app = composite();
    assert!(
        app.layers.base.data.composite_probe.is_some(),
        "a composite"
    );
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    let tile = app.layers.base.data.detail.tile.as_ref().expect("tile");
    assert!(tile.probe.is_some(), "the tile keeps its window");
    let [u0, v0, u1, v1] = tile.uv;
    let (x, y) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
    let (probe, cell) = crate::ui::hover::detail::tile_probe(&app, x, y).expect("under the tile");
    assert!(probe.sample(0, cell.0, cell.1).is_some(), "raw red values");
}

#[test]
fn tiles_use_their_plots_stretch() {
    let mut app = composite();
    rest_at(&mut app, 4.0);
    let tile = app.layers.base.data.detail.tile.clone().expect("tile");
    let mut stretch = app.layers.base.data.composite_stretch.clone();
    let stretch = stretch.as_mut().expect("the plot records its stretch");
    // The plot's red stretch, which the tile's own values would not give.
    let s = stretch.channel(0, &[], None, 255.0);
    let matrix = tile.matrix.as_ref().expect("values");
    let probe = tile.probe.as_ref().expect("window");
    let red = |px, py| probe.sample(0, px, py).expect("red");
    let values: Vec<f32> = (0..matrix.height)
        .flat_map(|py| (0..matrix.width).map(move |px| (px, py)))
        .map(|(px, py)| red(px, py))
        .collect();
    assert_ne!(s, crate::data::slicing::Stretch::of(&values, None, 255.0));
    let (r, _, _) = crate::data::slicing::composite::unpack_rgb(matrix.values[0]);
    let expected = ((red(0, 0) - s.offset) * s.scale).clamp(0.0, 255.0);
    assert!(
        (r - expected).abs() <= 1.0,
        "red {r}, plot's stretch {expected}"
    );
}

#[test]
fn other_bands_drop_the_tile() {
    let mut app = composite();
    rest_at(&mut app, 4.0);
    assert_eq!(tile_level(&app), Some(0));
    app.layers.base.composite.rgb_channels = [2, 1, 0];
    super::tests::frame_at(&mut app, PlotType::Heatmap, 4.0, 1.0);
    assert_eq!(tile_level(&app), None, "drawn with other bands");
}
