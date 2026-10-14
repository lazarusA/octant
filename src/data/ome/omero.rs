//! OMERO rendering settings (`omero`), read channel by channel.
//!
//! `ome_zarr_metadata` types only a channel's `color` and `window` and keeps
//! the rest (`label`, `active`, `inverted`, `family`, `coefficient`) as untyped
//! JSON, and fails a whole `omero` object over one malformed channel; channels
//! are therefore read here, leniently, from the raw JSON only.

use serde_json::Value;

use super::model::{ChannelFamily, ChannelWindow, OmeChannel, OmeImage};
use super::parse::json_text;
use crate::data::slicing::{DEFAULT_CHANNEL_COLORS, parse_hex_color};

/// Every channel of `omero`, in order.
pub(super) fn channels(omero: Option<&Value>) -> Vec<OmeChannel> {
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

/// Sets the image-wide rendering defaults (`omero.rdefs`) of `image`.
pub(super) fn apply_render_defaults(omero: Option<&Value>, image: &mut OmeImage) {
    let Some(rdefs) = omero.and_then(|o| o.get("rdefs")) else {
        return;
    };
    let index = |key: &str| rdefs.get(key).and_then(Value::as_u64).map(|v| v as usize);
    image.default_t = index("defaultT");
    image.default_z = index("defaultZ");
    image.greyscale = rdefs
        .get("model")
        .and_then(Value::as_str)
        .is_some_and(|m| m.trim().eq_ignore_ascii_case("greyscale"));
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
        label: c
            .get("label")
            .and_then(json_text)
            .or_else(|| c.get("name").and_then(json_text))
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
