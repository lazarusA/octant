//! Bricks per mode and data kind over the in-memory pyramid (`tests.rs`):
//! which modes guess occlusion, byte atlases, composite recoloring.

use super::tests::{plotted, rest};
use crate::app::OctantApp;

#[test]
fn opaque_label_surfaces_hide_what_lies_behind() {
    let mut app = plotted();
    let volume = &mut app.plot_configs.volume;
    volume.algorithm = 4;
    volume.transparency = false;
    assert!(app.occluding().is_some_and(|o| o.opaque), "opaque labels");
    app.plot_configs.volume.transparency = true;
    assert!(app.occluding().is_none(), "see-through labels hide nothing");
    app.plot_configs.volume.algorithm = 1;
    assert!(app.occluding().is_none(), "MIP reads every voxel");
}

#[test]
fn eight_bit_values_survive_the_byte_atlas_exactly() {
    use super::encode::{encode_value, to_unorm8};
    use super::placement::byte_offset;
    for (data_type, offset) in [("uint8", 0.0), ("|u1", 0.0), ("int8", -128.0)] {
        assert_eq!(byte_offset(data_type), Some(offset), "{data_type}");
        let values: Vec<f32> = (0..256).map(|k| k as f32 + offset).collect();
        let half: Vec<u16> = values
            .iter()
            .map(|&v| encode_value(v, offset, 255.0))
            .collect();
        let bytes = to_unorm8(&half);
        let back: Vec<f32> = bytes.iter().map(|&b| f32::from(b) + offset).collect();
        assert_eq!(back, values, "{data_type}");
        // The skip ranges decode what the GPU reads, not the f16 (7 would
        // be 6.9999 and a block of 7s skipped at cmin 7).
        let encoding = super::encode::Encoding {
            offset,
            scale: 255.0,
            byte: true,
        };
        let decoded: Vec<f32> = half.iter().map(|&h| encoding.decode(h)).collect();
        assert_eq!(decoded, values, "{data_type} ranges");
    }
    assert_eq!(byte_offset("uint16"), None);
    assert_eq!(byte_offset("float32"), None);
}

#[test]
fn only_whole_values_in_range_go_to_a_byte_atlas() {
    use super::placement::bytes_hold;
    assert!(bytes_hold(&[0.0, 17.0, 255.0], 0.0));
    assert!(bytes_hold(&[-128.0, 127.0], -128.0));
    assert!(!bytes_hold(&[0.0, 273.4], 0.0), "packed, unpacked to reals");
    assert!(!bytes_hold(&[1000.0], 0.0), "an add_offset past the byte");
    assert!(!bytes_hold(&[f32::NAN], 0.0), "missing voxels");
}

#[test]
fn composite_changes_recolor_only_once_they_rest() {
    use super::composite::{CompositeLod, colors_of};
    use crate::app::detail::SETTLE;
    let mut app = plotted();
    rest(&mut app);
    app.layers.base.composite.enabled = true;
    let colored = colors_of(&app);
    let block = crate::data::OctantBlock::new(
        "v".into(),
        vec![2, 4, 4, 4],
        ["c", "z", "y", "x"].map(String::from).to_vec(),
        vec![0; 4],
        std::sync::Arc::from(vec![1.0f32; 128]),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    )
    .expect("block");
    let lod = app
        .layers
        .base
        .data
        .volume_lod
        .lod
        .as_mut()
        .expect("bricked");
    lod.composite = Some(CompositeLod::channels(0, Vec::new(), &block, (3, 2, 1)));
    app.layers.base.data.volume_lod.colored = Some(colored);
    let built = |a: &OctantApp| a.layers.base.data.volume_lod.colored;
    assert_eq!(
        app.recolor_volume_lod(0.0),
        None,
        "unchanged: nothing to do"
    );
    // Dragging: every change restarts the wait, the bricks stay.
    for (t, ch) in [(0.0, 1), (0.1, 2), (0.2, 3)] {
        app.layers.base.composite.rgb_channels = [ch, 1, 2];
        assert!(app.recolor_volume_lod(t).is_some(), "waits at {t}");
        assert_eq!(built(&app), Some(colored), "kept at {t}");
    }
    assert!(app.recolor_volume_lod(0.2 + SETTLE / 2.0).is_some());
    assert_eq!(built(&app), Some(colored), "still resting");
    app.recolor_volume_lod(0.3 + SETTLE);
    assert_ne!(built(&app), Some(colored), "rebuilt once rested");
}

#[test]
fn turning_a_composite_on_rebuilds_the_bricks_at_once() {
    let mut app = plotted();
    rest(&mut app);
    // Accepting the block cleared its key: the plotted block is still found.
    assert!(app.layers.base.load.block_key.is_none());
    assert!(app.plotted_block().is_some(), "found by its request");
    let built = |a: &OctantApp| a.layers.base.data.volume_lod.colored;
    let before = built(&app);
    assert!(before.is_some(), "single-channel bricks");
    app.layers.base.composite.enabled = true;
    assert_eq!(app.recolor_volume_lod(0.0), None, "no wait for a toggle");
    assert_ne!(built(&app), before, "rebuilt for the composite");
}

#[test]
fn a_composite_switch_waits_for_its_block() {
    let mut app = plotted();
    rest(&mut app);
    let built = |a: &OctantApp| a.layers.base.data.volume_lod.colored;
    let before = built(&app);
    // The switch asked for the colour block, which has not arrived.
    let slice = app.layers.base.load.slice_request.take();
    app.layers.base.composite.enabled = true;
    assert!(app.recolor_volume_lod(0.0).is_some(), "waits for the block");
    assert_eq!(built(&app), before, "the bricks stay meanwhile");
    app.layers.base.load.slice_request = slice;
    assert_eq!(app.recolor_volume_lod(0.1), None);
    assert_ne!(built(&app), before, "rebuilt once it arrived");
}

#[test]
fn a_dense_plots_composite_settings_wait_to_rest() {
    let mut app = plotted();
    rest(&mut app);
    // The last build stayed dense (too many roots, say): no bricks to swap.
    app.layers.base.data.volume_lod.lod = None;
    let built = |a: &OctantApp| a.layers.base.data.volume_lod.colored;
    let before = built(&app);
    app.layers.base.composite.enabled = true;
    assert!(
        app.recolor_volume_lod(0.0).is_some(),
        "waits like any setting"
    );
    assert_eq!(built(&app), before, "nothing rebuilt while dragging");
}

#[test]
fn settings_waiting_on_a_block_nobody_reads_rebuild_without_bricks() {
    use super::composite::colors_of;
    let mut app = plotted();
    rest(&mut app);
    // A dense composite plot whose block left the cache, no read pending.
    app.layers.base.data.volume_lod.lod = None;
    app.layers.base.composite.enabled = true;
    app.layers.base.load.slice_request = None;
    app.layers.base.load.block_key = None;
    assert_eq!(app.recolor_volume_lod(0.0), None, "never polls for good");
    let state = &app.layers.base.data.volume_lod;
    assert_eq!(
        state.colored,
        Some(colors_of(&app)),
        "built in the new colors"
    );
    assert_eq!(app.recolor_volume_lod(1.0), None, "and stays settled");
}
