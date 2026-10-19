//! A window of a composite sliced with its plot's stretch draws the plot's
//! colors; sliced on its own it stretches over its own values instead.

use std::collections::HashMap;

use super::stretch::CompositeStretch;
use super::types::ChannelColorConfig;
use super::{slice_multichannel_composite_nd, slice_rgb_composite_nd};
use crate::data::octant_block::OctantBlock;

const BANDS: usize = 4;
const H: usize = 4;
const W: usize = 8;

/// A `[4, 4, 8]` block whose values run past 255 (so stretched over their
/// range), with CMYK inks when `cmyk`.
fn block(cmyk: bool) -> OctantBlock {
    let values: Vec<f32> = (0..BANDS * H * W).map(|i| (i * 37) as f32).collect();
    let attributes = if cmyk {
        HashMap::from([("photometric".to_string(), "cmyk".to_string())])
    } else {
        HashMap::new()
    };
    OctantBlock::new(
        "bands".into(),
        vec![BANDS, H, W],
        ["band", "y", "x"].map(String::from).to_vec(),
        vec![0, 0, 0],
        values,
        HashMap::new(),
        attributes,
    )
}

/// The right half of the top two rows.
const TILE_X: (usize, usize) = (4, 8);
const TILE_Y: (usize, usize) = (0, 2);

/// The plot's values over the tile's cells.
fn plot_over_tile(plot: &[f32]) -> Vec<f32> {
    (TILE_Y.0..TILE_Y.1)
        .flat_map(|y| (TILE_X.0..TILE_X.1).map(move |x| plot[y * W + x]))
        .collect()
}

fn rgb(
    b: &OctantBlock,
    x: (usize, usize),
    y: (usize, usize),
    s: &mut CompositeStretch,
) -> Vec<f32> {
    let channels = [Some(0), Some(2), Some(3)];
    let m = slice_rgb_composite_nd(b, 0, 2, 1, x, y, &[0, 0, 0], channels, 1, s);
    m.expect("composite").values.to_vec()
}

#[test]
fn rgb_and_cmyk_tiles_keep_their_plots_colors() {
    for cmyk in [false, true] {
        let b = block(cmyk);
        let mut stretch = CompositeStretch::default();
        let plot = rgb(&b, (0, W), (0, H), &mut stretch);
        let tile = rgb(&b, TILE_X, TILE_Y, &mut stretch.clone());
        assert_eq!(tile, plot_over_tile(&plot), "cmyk {cmyk}");
        let alone = rgb(&b, TILE_X, TILE_Y, &mut CompositeStretch::default());
        assert_ne!(alone, tile, "cmyk {cmyk}: its own range differs");
    }
}

#[test]
fn multichannel_tiles_keep_their_plots_colors_unless_windowed() {
    let b = block(false);
    let config = |index: usize, window| ChannelColorConfig {
        index,
        name: format!("ch{index}"),
        color_rgb: [[255, 0, 0], [0, 255, 0], [0, 0, 255]][index],
        visible: true,
        window,
        display: Default::default(),
    };
    let slice = |configs: &[ChannelColorConfig], x, y, s: &mut CompositeStretch| {
        let m = slice_multichannel_composite_nd(&b, 0, 2, 1, x, y, &[0, 0, 0], configs, 1, s);
        m.expect("composite").values.to_vec()
    };
    let configs: Vec<_> = (0..3).map(|i| config(i, None)).collect();
    let mut stretch = CompositeStretch::default();
    let plot = slice(&configs, (0, W), (0, H), &mut stretch);
    let tile = slice(&configs, TILE_X, TILE_Y, &mut stretch.clone());
    assert_eq!(tile, plot_over_tile(&plot));
    assert_ne!(
        slice(&configs, TILE_X, TILE_Y, &mut CompositeStretch::default()),
        tile
    );

    // A window decides the stretch, whatever was recorded.
    let windowed: Vec<_> = (0..3).map(|i| config(i, Some((0.0, 4000.0)))).collect();
    let recorded = slice(&windowed, TILE_X, TILE_Y, &mut stretch.clone());
    let fresh = slice(&windowed, TILE_X, TILE_Y, &mut CompositeStretch::default());
    assert_eq!(recorded, fresh);
}
