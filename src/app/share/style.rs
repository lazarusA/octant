//! The base layer's composite and color settings in a link:
//! `composite=on|off`, and while on `rgb=R,G,B` (its band slots) and
//! `channels=label:rrggbb[:a..b][:curve[^k]][:inv],...` (its ticked channel
//! rows), as the layer's `CompositeStyle` holds them whatever the dataset;
//! `range=a..b`, `scale=name[:param]`, `opacity`, `reverse=on`, and the
//! NaN and clip colors `nan`, `low`, `high` as `rrggbbaa` (left out: off).
//! Channels and bands are named as `value_text` names any dimension value.

use super::build::pair;
use crate::app::OctantApp;
use crate::app::VariableSelection;
use crate::app::actions::{
    AppAction, CURVES, ChannelSetting, ColorSetting, CompositeSetting, SCALES, hex_rgba, value_text,
};
use crate::app::layers::LayerId;
use crate::data::slicing::ChannelColorConfig;

impl OctantApp {
    /// `composite`, `rgb` and `channels`: the base layer's composite state,
    /// for a variable with a channel axis or a composite drawn.
    pub(super) fn composite_pairs(&self, sel: &VariableSelection, q: &mut String) {
        let id = LayerId::BASE;
        let composite = &self.layers.base.composite;
        let channel_axis = sel.variable_info().and_then(|v| v.channel_axis());
        if !self.layer_has_rgb_bands(id) || (channel_axis.is_none() && !composite.enabled) {
            return;
        }
        pair(q, "composite", if composite.enabled { "on" } else { "off" });
        let (Some(meta), Some(var), Some(c)) = (
            sel.metadata.as_ref(),
            sel.variable_info(),
            self.layer_channel_dim(id),
        ) else {
            return;
        };
        if !composite.enabled {
            return;
        }
        let text = |i: usize| value_text(meta, var, &sel.store_target, c, (i, false));
        // Slots past the bands (three slots, two channels) are no state to keep.
        let bands = self.layer_num_bands(id);
        if composite.rgb_channels.iter().all(|&b| b < bands) {
            pair(q, "rgb", &composite.rgb_channels.map(text).join(","));
        }
        let ticked: Vec<String> = composite
            .channel_configs
            .iter()
            .filter(|cfg| cfg.visible)
            .map(|cfg| channel_item(&text(cfg.index), cfg))
            .collect();
        pair(q, "channels", &ticked.join(","));
    }

    /// `range` (none for a composite, which draws no colormap) and the color
    /// settings that differ from the defaults.
    pub(super) fn style_pairs(&self, q: &mut String) {
        let s = &self.layers.base.color;
        let colormapped = !self.layers.base.composite.enabled;
        if colormapped && s.range_min.is_finite() && s.range_max.is_finite() {
            pair(q, "range", &format!("{}..{}", s.range_min, s.range_max));
        }
        let scale = SCALES
            .get(s.scale_type as usize)
            .copied()
            .unwrap_or("linear");
        match s.scale_type {
            0 => {}
            1 | 2 | 4 => pair(q, "scale", &format!("{scale}:{}", s.scale_param)),
            _ => pair(q, "scale", scale),
        }
        if s.opacity < 1.0 {
            pair(q, "opacity", &s.opacity.to_string());
        }
        if s.reversed {
            pair(q, "reverse", "on");
        }
        let colors = [
            ("nan", s.use_nan_color, s.nan_color),
            ("low", s.use_lowclip, s.lowclip_color),
            ("high", s.use_highclip, s.highclip_color),
        ];
        for (key, on, color) in colors {
            if on {
                pair(q, key, &hex_rgba(color));
            }
        }
    }
}

/// `label:rrggbb[:a..b][:curve[^k]][:inv]`.
fn channel_item(label: &str, cfg: &ChannelColorConfig) -> String {
    let [r, g, b] = cfg.color_rgb;
    let mut item = format!("{label}:{r:02x}{g:02x}{b:02x}");
    if let Some((a, b)) = cfg.window {
        item += &format!(":{a}..{b}");
    }
    let d = &cfg.display;
    let curve = CURVES.iter().find(|(f, _)| *f == d.family).map(|(_, n)| *n);
    match curve {
        Some("linear") | None => {}
        Some(name) if d.coefficient == 1.0 => item += &format!(":{name}"),
        Some(name) => item += &format!(":{name}^{}", d.coefficient),
    }
    if d.inverted {
        item += ":inv";
    }
    item
}

/// `SetComposite` from `composite`, `rgb` and `channels` (on when only
/// `rgb` or `channels` is given).
pub(super) fn composite_action(
    composite: Option<&str>,
    rgb: Option<&str>,
    channels: Option<&str>,
) -> Option<AppAction> {
    if composite.is_none() && rgb.is_none() && channels.is_none() {
        return None;
    }
    let list = |text: Option<&str>| -> Vec<String> {
        let items = text.unwrap_or("").split(',').map(str::trim);
        items
            .filter(|i| !i.is_empty())
            .map(str::to_string)
            .collect()
    };
    let composite = CompositeSetting {
        enabled: composite.is_none_or(|c| !c.trim().eq_ignore_ascii_case("off")),
        rgb: list(rgb),
        channels: list(channels)
            .iter()
            .filter_map(|c| channel_setting(c))
            .collect(),
    };
    Some(AppAction::SetComposite { composite })
}

/// One `channels` item; its parts after the label are told apart by form.
fn channel_setting(item: &str) -> Option<ChannelSetting> {
    let mut parts = item.split(':');
    let mut setting = ChannelSetting {
        channel: parts.next()?.trim().to_string(),
        ..ChannelSetting::default()
    };
    for part in parts.map(str::trim) {
        if part.eq_ignore_ascii_case("inv") {
            setting.inverted = true;
        } else if let Some((a, b)) = part.split_once("..") {
            setting.window_from = a.trim().parse().ok();
            setting.window_to = b.trim().parse().ok();
        } else if part.len() == 6 && part.bytes().all(|c| c.is_ascii_hexdigit()) {
            setting.color = Some(part.to_string());
        } else {
            let (curve, k) = part.split_once('^').unwrap_or((part, ""));
            setting.curve = Some(curve.to_string());
            setting.coefficient = k.parse().ok();
        }
    }
    (!setting.channel.is_empty()).then_some(setting)
}

/// `SetColorStyle` from the color keys, when any is given.
pub(super) fn style_action(last: impl Fn(&str) -> Option<String>) -> Option<AppAction> {
    let (scale, scale_param) = match last("scale") {
        Some(s) => {
            let (name, param) = s.split_once(':').unwrap_or((&s, ""));
            (Some(name.to_string()), param.trim().parse().ok())
        }
        None => (None, None),
    };
    let style = ColorSetting {
        scale,
        scale_param,
        opacity: last("opacity").and_then(|o| o.trim().parse().ok()),
        reversed: last("reverse").map(|r| !r.trim().eq_ignore_ascii_case("off")),
        nan_color: last("nan"),
        low_color: last("low"),
        high_color: last("high"),
    };
    (style != ColorSetting::default()).then_some(AppAction::SetColorStyle { style })
}
