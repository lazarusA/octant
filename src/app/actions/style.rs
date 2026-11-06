//! The base layer's composite and color settings, set as its Settings
//! controls set them (share links' `rgb`, `channels`, `scale`, `opacity`,
//! `reverse`, `nan`, `low`, `high`).

use serde::{Deserialize, Serialize};

use super::{Done, Failure, Outcome};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::ome::ChannelFamily;
use crate::data::slicing::ChannelColorConfig;

/// Color scales by name, in `ColorStyle::scale_type` order.
pub const SCALES: [&str; 5] = ["linear", "log", "symlog", "sqrt", "exponential"];

/// Channel curves by name, as OMERO names its families.
pub const CURVES: [(ChannelFamily, &str); 4] = [
    (ChannelFamily::Linear, "linear"),
    (ChannelFamily::Logarithmic, "logarithmic"),
    (ChannelFamily::Exponential, "exponential"),
    (ChannelFamily::Polynomial, "polynomial"),
];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CompositeSetting {
    /// Draws the variable's bands or channels as one color image.
    pub enabled: bool,
    /// The bands drawn red, green and blue, each a value of the channel
    /// dimension as in `SetDim`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rgb: Vec<String>,
    /// The channels a multi-channel image draws, the others hidden (none
    /// when empty); left out, the channels are kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<ChannelSetting>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ChannelSetting {
    /// The channel, a value of the channel dimension as in `SetDim`.
    pub channel: String,
    /// Its tint, `rrggbb`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// The values shown from black to full tint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_from: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_to: Option<f32>,
    /// linear, logarithmic, exponential or polynomial.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curve: Option<String>,
    /// The curve's exponent or growth, positive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coefficient: Option<f32>,
    #[serde(default)]
    pub inverted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ColorSetting {
    /// linear, log, symlog, sqrt or exponential.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    /// The scale's parameter (log, symlog and exponential).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_param: Option<f32>,
    /// Opacity of the colors, 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    /// Samples the colormap from its end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversed: Option<bool>,
    /// The color of missing values, `rrggbbaa`, or `off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nan_color: Option<String>,
    /// The color below the range, `rrggbbaa`, or `off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low_color: Option<String>,
    /// The color above the range, `rrggbbaa`, or `off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high_color: Option<String>,
}

impl OctantApp {
    /// The base layer's composite state, as its Settings toggle, band
    /// selects and channel rows set it, whatever the dataset.
    pub(super) fn set_composite(&mut self, s: &CompositeSetting) -> Outcome {
        let id = LayerId::BASE;
        if s.enabled && !self.layer_has_rgb_bands(id) {
            return Err(no_value("rgb", "on"));
        }
        let rgb = s
            .rgb
            .iter()
            .map(|v| self.channel_index(v))
            .collect::<Result<Vec<_>, _>>()?;
        let rgb = match rgb[..] {
            [] => None,
            [r, g, b] => Some([r, g, b]),
            _ => return Err(no_value("rgb", &s.rgb.join(","))),
        };
        let channels = s.channels.as_ref().map(|list| {
            let found = list
                .iter()
                .map(|c| Ok((self.channel_index(&c.channel)?, c)));
            found.collect::<Result<Vec<_>, Failure>>()
        });
        let channels = channels.transpose()?;
        let Some(layer) = self.layers.get_mut(id) else {
            return Err(Failure::NoDataset);
        };
        let composite = &mut layer.composite;
        let before = composite.clone();
        composite.enabled = s.enabled;
        if let Some(rgb) = rgb {
            composite.rgb_channels = rgb;
        }
        if let Some(channels) = channels {
            let configs = &mut composite.channel_configs;
            configs.iter_mut().for_each(|cfg| cfg.visible = false);
            for (i, c) in channels {
                if let Some(cfg) = configs.get_mut(i) {
                    apply_channel(cfg, c)?;
                }
            }
        }
        // As the Settings controls reload: the toggle and band selects the
        // layer, a channel row's display through `channel_display_changed`.
        let reads = (composite.enabled, composite.rgb_channels);
        if reads != (before.enabled, before.rgb_channels) {
            self.load_layer_block(id);
        } else if composite.channel_configs != before.channel_configs {
            self.channel_display_changed(id);
        }
        Ok(Done::CompositeSet)
    }

    /// The base layer's color scale, opacity, reversal and NaN and clip
    /// colors that `s` names; the others are kept.
    pub(super) fn set_color_style(&mut self, s: &ColorSetting) -> Outcome {
        let scale = match s.scale.as_deref() {
            Some(name) => Some(
                SCALES
                    .iter()
                    .position(|k| k.eq_ignore_ascii_case(name.trim()))
                    .ok_or_else(|| no_value("scale", name))?,
            ),
            None => None,
        };
        let nan = switchable(s.nan_color.as_deref(), "nan")?;
        let low = switchable(s.low_color.as_deref(), "low")?;
        let high = switchable(s.high_color.as_deref(), "high")?;
        let style = &mut self.layers.base.color;
        if let Some(t) = scale {
            style.scale_type = t as u32;
        }
        if let Some(p) = s.scale_param.filter(|p| p.is_finite()) {
            style.scale_param = p.clamp(0.0001, 100.0);
        }
        if let Some(o) = s.opacity.filter(|o| o.is_finite()) {
            style.opacity = o.clamp(0.0, 1.0);
        }
        if let Some(r) = s.reversed {
            style.reversed = r;
        }
        let slots = [
            (nan, &mut style.use_nan_color, &mut style.nan_color),
            (low, &mut style.use_lowclip, &mut style.lowclip_color),
            (high, &mut style.use_highclip, &mut style.highclip_color),
        ];
        for (asked, on, color) in slots {
            match asked {
                Some(Some(c)) => (*on, *color) = (true, c),
                Some(None) => *on = false,
                None => {}
            }
        }
        Ok(Done::ColorStyleSet)
    }

    /// The index `value` names along the base layer's channel dimension.
    fn channel_index(&mut self, value: &str) -> Result<usize, Failure> {
        let Some(c) = self.layer_channel_dim(LayerId::BASE) else {
            return Err(no_value("channel", value));
        };
        self.await_coordinates(c)?;
        self.index_for(c, value, false)
    }
}

/// Sets `cfg` drawn as `c` says; its window, curve and coefficient only when
/// given.
fn apply_channel(cfg: &mut ChannelColorConfig, c: &ChannelSetting) -> Result<(), Failure> {
    cfg.visible = true;
    if let Some(hex) = c.color.as_deref() {
        let rgb = crate::data::slicing::parse_hex_color(hex);
        cfg.color_rgb = rgb.ok_or_else(|| no_value(&c.channel, hex))?;
    }
    if let (Some(a), Some(b)) = (c.window_from, c.window_to)
        && a.is_finite()
        && b.is_finite()
    {
        cfg.window = Some((a, b));
    }
    if let Some(curve) = c.curve.as_deref() {
        let found = CURVES
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(curve.trim()));
        cfg.display.family = found.ok_or_else(|| no_value(&c.channel, curve))?.0;
    }
    if let Some(k) = c.coefficient.filter(|k| k.is_finite() && *k > 0.0) {
        cfg.display.coefficient = k;
    }
    cfg.display.inverted = c.inverted;
    Ok(())
}

/// A color as `rrggbbaa` (`rrggbb` opaque), `Some(None)` for `off`.
fn switchable(text: Option<&str>, what: &str) -> Result<Option<Option<[f32; 4]>>, Failure> {
    let Some(text) = text.map(str::trim) else {
        return Ok(None);
    };
    if text.eq_ignore_ascii_case("off") {
        return Ok(Some(None));
    }
    rgba(text)
        .map(|c| Some(Some(c)))
        .ok_or_else(|| no_value(what, text))
}

/// `rrggbb` or `rrggbbaa` as RGBA fractions.
fn rgba(hex: &str) -> Option<[f32; 4]> {
    let hex = hex.trim().trim_start_matches('#');
    if !hex.is_ascii() || !matches!(hex.len(), 6 | 8) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    let alpha = if hex.len() == 8 { byte(6)? } else { 255 };
    let [r, g, b] = [byte(0)?, byte(2)?, byte(4)?];
    Some([r, g, b, alpha].map(|c| f32::from(c) / 255.0))
}

/// `[r, g, b, a]` fractions as `rrggbbaa`.
pub(crate) fn hex_rgba(color: [f32; 4]) -> String {
    let [r, g, b, a] = color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("{r:02x}{g:02x}{b:02x}{a:02x}")
}

fn no_value(dim: &str, value: &str) -> Failure {
    Failure::NoValue {
        dim: dim.to_string(),
        value: value.to_string(),
    }
}
