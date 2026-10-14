//! OMERO rendering settings (`omero`), read channel by channel.
//!
//! `ome_zarr_metadata` types only a channel's `color` and `window` and keeps
//! the rest (`label`, `active`, `inverted`, `family`, `coefficient`) as untyped
//! JSON, and fails a whole `omero` object over one malformed channel; channels
//! are therefore read here, leniently, from the raw JSON only.

use serde_json::Value;

use super::model::{ChannelFamily, ChannelWindow, OmeChannel};
use crate::data::slicing::{DEFAULT_CHANNEL_COLORS, parse_hex_color};

/// The image-wide rendering defaults (`omero.rdefs`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderDefaults {
    pub default_t: Option<usize>,
    pub default_z: Option<usize>,
    /// `model: "greyscale"`.
    pub greyscale: bool,
}

/// Every channel of `omero`, in order.
pub fn channels(omero: Option<&Value>) -> Vec<OmeChannel> {
    let Some(list) = omero
        .and_then(|o| o.get("channels"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    list.iter()
        .enumerate()
        .map(|(i, c)| channel(c, i))
        .collect()
}

/// The `rdefs` of `omero`.
pub fn render_defaults(omero: Option<&Value>) -> RenderDefaults {
    let Some(rdefs) = omero.and_then(|o| o.get("rdefs")) else {
        return RenderDefaults::default();
    };
    let index = |key: &str| rdefs.get(key).and_then(Value::as_u64).map(|v| v as usize);
    RenderDefaults {
        default_t: index("defaultT"),
        default_z: index("defaultZ"),
        greyscale: rdefs
            .get("model")
            .and_then(Value::as_str)
            .is_some_and(|m| m.trim().eq_ignore_ascii_case("greyscale")),
    }
}

fn text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn number(v: Option<&Value>) -> Option<f32> {
    v.and_then(Value::as_f64).map(|n| n as f32)
}

fn channel(c: &Value, index: usize) -> OmeChannel {
    let color = c
        .get("color")
        .and_then(Value::as_str)
        .and_then(parse_hex_color)
        .unwrap_or(DEFAULT_CHANNEL_COLORS[index % DEFAULT_CHANNEL_COLORS.len()]);
    OmeChannel {
        label: text(c.get("label"))
            .or_else(|| text(c.get("name")))
            .unwrap_or_default(),
        color,
        active: c.get("active").and_then(Value::as_bool).unwrap_or(true),
        window: c.get("window").and_then(window),
        inverted: c.get("inverted").and_then(Value::as_bool).unwrap_or(false),
        family: c
            .get("family")
            .and_then(Value::as_str)
            .map(ChannelFamily::parse)
            .unwrap_or_default(),
        coefficient: number(c.get("coefficient"))
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(1.0),
    }
}

/// A window needs `start` and `end`; `min` and `max` default to them.
fn window(w: &Value) -> Option<ChannelWindow> {
    let start = number(w.get("start"))?;
    let end = number(w.get("end"))?;
    if !start.is_finite() || !end.is_finite() {
        return None;
    }
    let min = number(w.get("min"))
        .filter(|v| v.is_finite())
        .unwrap_or(start);
    let max = number(w.get("max"))
        .filter(|v| v.is_finite())
        .unwrap_or(end);
    Some(ChannelWindow {
        min: min.min(start),
        max: max.max(end),
        start,
        end,
    })
}
