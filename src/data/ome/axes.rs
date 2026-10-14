//! OME-Zarr axes: their kind (space, time, channel) and physical unit.

use ome_zarr_metadata::v0_5::{Axis, AxisType, AxisUnit};

/// What an axis measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisKind {
    Space,
    Time,
    Channel,
    /// A custom or missing `type`.
    Other,
}

/// One axis of an OME-Zarr image.
#[derive(Debug, Clone, PartialEq)]
pub struct OmeAxis {
    pub name: String,
    pub kind: AxisKind,
    /// The unit's spec name (`"micrometer"`), when given.
    pub unit: Option<String>,
}

impl OmeAxis {
    /// The axis described by `axis`; a missing `type` is guessed from the name.
    pub fn from_spec(axis: &Axis) -> Self {
        let kind = match &axis.r#type {
            Some(AxisType::Space) => AxisKind::Space,
            Some(AxisType::Time) => AxisKind::Time,
            Some(AxisType::Channel) => AxisKind::Channel,
            Some(_) => AxisKind::Other,
            None => kind_from_name(&axis.name),
        };
        let unit = axis.unit.as_ref().and_then(unit_name);
        Self {
            name: axis.name.clone(),
            kind,
            unit,
        }
    }

    /// A short unit symbol (`µm`, `s`), or the unit's own name when it has none.
    pub fn unit_symbol(&self) -> Option<&str> {
        let unit = self.unit.as_deref()?;
        Some(unit_symbol(unit).unwrap_or(unit))
    }
}

/// The axis kind its usual name implies (`t`, `c`, `z`, `y`, `x`).
pub fn kind_from_name(name: &str) -> AxisKind {
    let n = name.trim();
    if n.eq_ignore_ascii_case("t") || n.eq_ignore_ascii_case("time") {
        AxisKind::Time
    } else if n.eq_ignore_ascii_case("c") || n.eq_ignore_ascii_case("channel") {
        AxisKind::Channel
    } else if ["x", "y", "z"].iter().any(|s| n.eq_ignore_ascii_case(s)) {
        AxisKind::Space
    } else {
        AxisKind::Other
    }
}

/// Default axis names for metadata (0.1 and 0.2) that names none: the trailing
/// `t, c, z, y, x` axes of the rank.
pub fn default_axis_names(rank: usize) -> Vec<String> {
    const TCZYX: [&str; 5] = ["t", "c", "z", "y", "x"];
    if rank <= TCZYX.len() {
        TCZYX[TCZYX.len() - rank..]
            .iter()
            .map(|s| (*s).to_string())
            .collect()
    } else {
        (0..rank).map(|i| format!("dim_{i}")).collect()
    }
}

fn unit_name(unit: &AxisUnit) -> Option<String> {
    match serde_json::to_value(unit).ok()? {
        serde_json::Value::String(s) if !s.trim().is_empty() => Some(s),
        _ => None,
    }
}

/// The spec's units: name, short symbol, and meters per unit for lengths.
/// Units without a symbol keep their name.
const UNITS: &[(&str, &str, Option<f64>)] = &[
    ("angstrom", "Å", Some(1e-10)),
    ("attometer", "am", Some(1e-18)),
    ("centimeter", "cm", Some(1e-2)),
    ("decimeter", "dm", Some(1e-1)),
    ("femtometer", "fm", Some(1e-15)),
    ("foot", "ft", Some(0.3048)),
    ("inch", "in", Some(0.0254)),
    ("kilometer", "km", Some(1e3)),
    ("meter", "m", Some(1.0)),
    ("micrometer", "µm", Some(1e-6)),
    ("micron", "µm", Some(1e-6)),
    ("mile", "mi", Some(1609.344)),
    ("millimeter", "mm", Some(1e-3)),
    ("nanometer", "nm", Some(1e-9)),
    ("picometer", "pm", Some(1e-12)),
    ("yard", "yd", Some(0.9144)),
    ("attosecond", "as", None),
    ("centisecond", "cs", None),
    ("day", "d", None),
    ("decisecond", "ds", None),
    ("femtosecond", "fs", None),
    ("hour", "h", None),
    ("microsecond", "µs", None),
    ("millisecond", "ms", None),
    ("minute", "min", None),
    ("nanosecond", "ns", None),
    ("picosecond", "ps", None),
    ("second", "s", None),
];

fn unit(name: &str) -> Option<&'static (&'static str, &'static str, Option<f64>)> {
    let name = name.trim();
    UNITS.iter().find(|(n, ..)| n.eq_ignore_ascii_case(name))
}

/// The short symbol of a unit's spec name (`"micrometer"` -> `"µm"`).
pub fn unit_symbol(name: &str) -> Option<&'static str> {
    unit(name).map(|u| u.1)
}

/// Meters in one unit of length, for scale bars across units.
pub fn meters_per_unit(name: &str) -> Option<f64> {
    unit(name)?.2
}
