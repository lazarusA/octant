//! Unit tests for RGB, CMYK, and multi-channel composite slicing.

use std::collections::HashMap;
use std::sync::Arc;

use super::multichannel::slice_multichannel_composite_nd;
use super::rgb::slice_rgb_composite;
use super::types::{ChannelColorConfig, DEFAULT_CHANNEL_COLORS, parse_hex_color};

use super::utils::{pack_rgb, unpack_rgb};
use crate::data::octant_block::OctantBlock;

#[test]
fn test_rgb_pack_unpack_fidelity() {
    let test_colors = [
        (255.0, 255.0, 255.0),
        (255.0, 254.0, 255.0),
        (254.0, 255.0, 255.0),
        (255.0, 255.0, 254.0),
        (0.0, 0.0, 0.0),
        (255.0, 0.0, 0.0),
        (0.0, 255.0, 0.0),
        (0.0, 0.0, 255.0),
        (128.0, 128.0, 128.0),
    ];

    for (r, g, b) in test_colors {
        let packed_f32 = pack_rgb(r, g, b);
        let (unpacked_r, unpacked_g, unpacked_b) = unpack_rgb(packed_f32);

        assert_eq!(
            (unpacked_r, unpacked_g, unpacked_b),
            (r, g, b),
            "Fidelity mismatch for color ({r}, {g}, {b})"
        );
    }
}

#[test]
fn test_hex_color_parsing() {
    assert_eq!(parse_hex_color("0000FF"), Some([0, 0, 255]));
    assert_eq!(parse_hex_color("#FFFF00"), Some([255, 255, 0]));
    assert_eq!(parse_hex_color("  #00FF00  "), Some([0, 255, 0]));
    assert_eq!(parse_hex_color("invalid"), None);
    assert_eq!(parse_hex_color("12345"), None);
    assert_eq!(DEFAULT_CHANNEL_COLORS.len(), 8);
}

#[test]
fn test_multichannel_additive_composite() {
    // 2 channels: Ch 0 (Blue [0, 0, 255]), Ch 1 (Yellow [255, 255, 0])
    // Shape: [2, 2, 2] (channels, y, x)
    // Ch 0 values: [1.0, 0.0, 0.0, 1.0] (scaled 0..1)
    // Ch 1 values: [0.0, 1.0, 0.0, 1.0] (scaled 0..1)
    let values: Arc<[f32]> = Arc::from(vec![
        // Ch 0 (Blue)
        1.0, 0.0, 0.0, 1.0, // Ch 1 (Yellow)
        0.0, 1.0, 0.0, 1.0,
    ]);

    let block = OctantBlock::new(
        "test_fluor".to_string(),
        vec![2, 2, 2],
        vec!["c".to_string(), "y".to_string(), "x".to_string()],
        vec![0, 0, 0],
        values,
        HashMap::new(),
        HashMap::new(),
    );

    let configs = vec![
        ChannelColorConfig {
            index: 0,
            name: "LaminB1".to_string(),
            color_rgb: [0, 0, 255], // Blue
            visible: true,
            window: Some((0.0, 1.0)),
        },
        ChannelColorConfig {
            index: 1,
            name: "Dapi".to_string(),
            color_rgb: [255, 255, 0], // Yellow
            visible: true,
            window: Some((0.0, 1.0)),
        },
    ];

    let composite = slice_multichannel_composite_nd(
        &block,
        0, // c_dim
        2, // x_dim
        1, // y_dim
        (0, 2),
        (0, 2),
        &[0, 0, 0],
        &configs,
        1,
    )
    .expect("multichannel composite should succeed");

    assert_eq!(composite.width, 2);
    assert_eq!(composite.height, 2);
    assert_eq!(composite.values.len(), 4);

    // Pixel 0 (Ch0=1, Ch1=0) -> Blue: (0, 0, 255)
    let p0 = unpack_rgb(composite.values[0]);
    assert_eq!(p0, (0.0, 0.0, 255.0));

    // Pixel 1 (Ch0=0, Ch1=1) -> Yellow: (255, 255, 0)
    let p1 = unpack_rgb(composite.values[1]);
    assert_eq!(p1, (255.0, 255.0, 0.0));

    // Pixel 2 (Ch0=0, Ch1=0) -> Black: (0, 0, 0)
    let p2 = unpack_rgb(composite.values[2]);
    assert_eq!(p2, (0.0, 0.0, 0.0));

    // Pixel 3 (Ch0=1, Ch1=1) -> Additive Blue + Yellow = White (255, 255, 255)
    let p3 = unpack_rgb(composite.values[3]);
    assert_eq!(p3, (255.0, 255.0, 255.0));
}

#[test]
fn test_standard_rgb_composite_preserved() {
    let values: Arc<[f32]> = Arc::from(vec![
        // R plane
        255.0, 0.0, // G plane
        0.0, 255.0, // B plane
        0.0, 0.0,
    ]);

    let block = OctantBlock::new(
        "test_rgb".to_string(),
        vec![3, 1, 2],
        vec!["band".to_string(), "y".to_string(), "x".to_string()],
        vec![0, 0, 0],
        values,
        HashMap::new(),
        HashMap::new(),
    );

    let composite =
        slice_rgb_composite(&block, [0, 1, 2], 1).expect("standard rgb composite should succeed");

    assert_eq!(composite.width, 2);
    assert_eq!(composite.height, 1);
    let p0 = unpack_rgb(composite.values[0]);
    assert_eq!(p0, (255.0, 0.0, 0.0));
    let p1 = unpack_rgb(composite.values[1]);
    assert_eq!(p1, (0.0, 255.0, 0.0));
}
