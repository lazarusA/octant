//! One cell of a multi-channel composite, blended as the whole plane is: the
//! hover's color where the GPU blends the plane (`plots::channels`).

use super::probe::CompositeProbe;
use super::stretch::{CompositeStretch, Stretch};
use super::types::{ChannelColorConfig, ChannelCurve};
use super::utils::{drawn_in_config_order, normalize_channel_value, pack_rgb};

/// Adds `raw` (not NaN), stretched by `s`, through `curve` and in `tint`.
#[inline]
pub(super) fn add_tinted(
    rgb: &mut [f32; 3],
    raw: f32,
    s: Stretch,
    curve: &ChannelCurve,
    tint: [f32; 3],
) {
    let norm = curve.map(normalize_channel_value(
        raw, s.scale, s.offset, s.is_i8, 1.0,
    ));
    for (acc, t) in rgb.iter_mut().zip(tint) {
        *acc += norm * t;
    }
}

/// `rgb` packed into 24-bit TrueColor, NaN when no channel had a value.
#[inline]
pub(super) fn pack_or_nan([r, g, b]: [f32; 3], valid: bool) -> f32 {
    match valid {
        true => pack_rgb(
            r.clamp(0.0, 255.0),
            g.clamp(0.0, 255.0),
            b.clamp(0.0, 255.0),
        ),
        false => f32::NAN,
    }
}

/// The packed color of cell `(px, py)` of `probe`'s window: its drawn
/// channels from `configs`, those without a window stretched as `stretch`
/// recorded.
pub fn blend_pixel(
    probe: &CompositeProbe,
    configs: &[ChannelColorConfig],
    stretch: &CompositeStretch,
    (px, py): (usize, usize),
) -> f32 {
    blend_samples(configs, stretch, probe.channel_span(), |c| {
        probe.sample(c, px, py)
    })
}

/// The packed color of one cell whose raw value of channel `c` (of the
/// `count` from `c_start`) is `sample(c)`: its drawn channels from
/// `configs`, those without a window stretched as `stretch` recorded.
pub fn blend_samples(
    configs: &[ChannelColorConfig],
    stretch: &CompositeStretch,
    (c_start, count): (usize, usize),
    sample: impl Fn(usize) -> Option<f32>,
) -> f32 {
    let (mut rgb, mut valid) = ([0.0; 3], false);
    for (cfg, _) in drawn_in_config_order(configs, c_start, count) {
        let raw = sample(cfg.index).filter(|v| !v.is_nan());
        let s = stretch.of_channel(cfg.index, cfg.window);
        if let (Some(raw), Some(s)) = (raw, s) {
            add_tinted(
                &mut rgb,
                raw,
                s,
                &cfg.display.curve(),
                cfg.color_rgb.map(f32::from),
            );
            valid = true;
        }
    }
    pack_or_nan(rgb, valid)
}
