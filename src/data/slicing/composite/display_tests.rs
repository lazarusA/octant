//! Channel display curves, inversion and auto-contrast quantiles.

use std::collections::HashMap;

use super::{
    ChannelColorConfig, ChannelDisplay, CompositeProbe, slice_multichannel_composite_nd, unpack_rgb,
};
use crate::data::octant_block::OctantBlock;
use crate::data::ome::ChannelFamily;

fn display(family: ChannelFamily, coefficient: f32) -> ChannelDisplay {
    ChannelDisplay {
        family,
        coefficient,
        ..Default::default()
    }
}

#[test]
fn every_curve_keeps_its_ends_and_rises() {
    for family in [
        ChannelFamily::Linear,
        ChannelFamily::Logarithmic,
        ChannelFamily::Exponential,
        ChannelFamily::Polynomial,
    ] {
        for c in [0.5, 1.0, 3.0] {
            let d = display(family, c);
            assert!(d.map(0.0).abs() < 1e-6, "{family:?} {c}");
            assert!((d.map(1.0) - 1.0).abs() < 1e-5, "{family:?} {c}");
            // OMERO's exponential stays at 0 until near the top for k > 1.
            assert!(d.map(0.25) <= d.map(0.75), "{family:?} {c}");
            assert!(d.map(0.75) < d.map(1.0), "{family:?} {c}");
        }
    }
    assert_eq!(display(ChannelFamily::Polynomial, 2.0).map(0.5), 0.25);
}

#[test]
fn curves_match_omero_quantization() {
    // `omero-renderer` maps the window onto 0..=255, then normalizes the
    // family's function between its values at 0 and 255.
    let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
    let log = display(ChannelFamily::Logarithmic, 3.0);
    assert!(close(log.map(0.5), 0.874_911_7), "k is ignored");
    assert_eq!(log.map(0.002), 0.0, "below 1 reads as ln 1");
    let exp = display(ChannelFamily::Exponential, 0.5);
    assert!(close(exp.map(0.5), 0.009_305_571));
}

#[test]
fn inversion_follows_the_curve() {
    let d = ChannelDisplay {
        inverted: true,
        ..display(ChannelFamily::Polynomial, 2.0)
    };
    assert_eq!(d.map(0.0), 1.0);
    assert_eq!(d.map(0.5), 0.75);
    assert_eq!(d.map(2.0), 0.0);
}

/// One channel `(c, y, x) = (1, 1, 2)` holding 0 and 10.
fn ramp() -> OctantBlock {
    OctantBlock::new(
        "ramp".into(),
        vec![1, 1, 2],
        vec!["c".into(), "y".into(), "x".into()],
        vec![0, 0, 0],
        vec![0.0, 10.0],
        HashMap::new(),
        HashMap::new(),
    )
}

#[test]
fn inverted_channels_light_up_where_data_is_low() {
    let mut cfg = ChannelColorConfig::new(0, "a".into(), [255, 255, 255]);
    cfg.window = Some((0.0, 10.0));
    cfg.display.inverted = true;
    let configs = [cfg, ChannelColorConfig::new(1, String::new(), [0, 0, 0])];
    let m =
        slice_multichannel_composite_nd(&ramp(), 0, 2, 1, (0, 2), (0, 1), &[0, 0, 0], &configs, 1)
            .expect("composite");
    assert_eq!(unpack_rgb(m.values[0]), (255.0, 255.0, 255.0));
    assert_eq!(unpack_rgb(m.values[1]), (0.0, 0.0, 0.0));
}

#[test]
fn quantiles_read_the_composite_window() {
    let values: Vec<f32> = (0..100).map(|v| v as f32).chain([f32::NAN]).collect();
    let block = OctantBlock::new(
        "q".into(),
        vec![1, 1, 101],
        vec!["c".into(), "y".into(), "x".into()],
        vec![0, 0, 0],
        values,
        HashMap::new(),
        HashMap::new(),
    );
    let probe = CompositeProbe::new(&block, 0, 2, 1, (0, 101), (0, 1), &[0, 0, 0]);
    assert_eq!(probe.channel_quantiles(0, 0.0, 1.0), Some((0.0, 99.0)));
    let (lo, hi) = probe.channel_quantiles(0, 0.1, 0.9).expect("quantiles");
    assert!((9.0..=11.0).contains(&lo) && (88.0..=90.0).contains(&hi));
    assert_eq!(probe.channel_quantiles(3, 0.0, 1.0), None);
}
