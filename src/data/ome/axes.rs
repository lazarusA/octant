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

/// Short symbols of the spec's units. Units without one keep their name.
const UNIT_SYMBOLS: &[(&str, &str)] = &[
    ("angstrom", "Å"),
    ("attometer", "am"),
    ("centimeter", "cm"),
    ("decimeter", "dm"),
    ("femtometer", "fm"),
    ("foot", "ft"),
    ("inch", "in"),
    ("kilometer", "km"),
    ("meter", "m"),
    ("micrometer", "µm"),
    ("micron", "µm"),
    ("mile", "mi"),
    ("millimeter", "mm"),
    ("nanometer", "nm"),
    ("picometer", "pm"),
    ("yard", "yd"),
    ("attosecond", "as"),
    ("centisecond", "cs"),
    ("day", "d"),
    ("decisecond", "ds"),
    ("femtosecond", "fs"),
    ("hour", "h"),
    ("microsecond", "µs"),
    ("millisecond", "ms"),
    ("minute", "min"),
    ("nanosecond", "ns"),
    ("picosecond", "ps"),
    ("second", "s"),
];

/// The short symbol of a unit's spec name (`"micrometer"` -> `"µm"`).
pub fn unit_symbol(unit: &str) -> Option<&'static str> {
    let unit = unit.trim();
    UNIT_SYMBOLS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(unit))
        .map(|(_, symbol)| *symbol)
}

/// Meters in one unit of length, for scale bars across units.
pub fn meters_per_unit(unit: &str) -> Option<f64> {
    const LENGTHS: &[(&str, f64)] = &[
        ("angstrom", 1e-10),
        ("picometer", 1e-12),
        ("nanometer", 1e-9),
        ("micrometer", 1e-6),
        ("micron", 1e-6),
        ("millimeter", 1e-3),
        ("centimeter", 1e-2),
        ("decimeter", 1e-1),
        ("meter", 1.0),
        ("kilometer", 1e3),
        ("inch", 0.0254),
        ("foot", 0.3048),
        ("yard", 0.9144),
        ("mile", 1609.344),
    ];
    let unit = unit.trim();
    LENGTHS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(unit))
        .map(|(_, m)| *m)
}

/// The dimension names of an OME-Zarr level array of rank `rank`, from the
/// attributes it inherits from its image group (`multiscales`, or 0.5's
/// `ome`, as JSON text); `None` for arrays outside an OME-Zarr image.
pub fn block_dimension_names(
    attributes: &std::collections::HashMap<String, String>,
    rank: usize,
) -> Option<Vec<String>> {
    let parse = |key: &str| serde_json::from_str::<serde_json::Value>(attributes.get(key)?).ok();
    let multiscales = parse("multiscales")
        .or_else(|| parse("ome").and_then(|ome| ome.get("multiscales").cloned()))?;
    let axes = multiscales.get(0)?.get("axes").and_then(|a| a.as_array());
    let names: Vec<String> = axes
        .into_iter()
        .flatten()
        .filter_map(|a| match a {
            serde_json::Value::String(s) => Some(s.clone()),
            other => other.get("name")?.as_str().map(str::to_string),
        })
        .collect();
    Some(if names.len() == rank {
        names
    } else {
        default_axis_names(rank)
    })
}

/// Block attribute marking values that are label ids stored as `f32` bits.
pub const LABEL_BITS_ATTRIBUTE: &str = "octant_label_bits";

/// Whether an array's inherited attributes make it a label image
/// (`image-label`, free or in 0.5's `ome`).
pub fn is_label_image(attributes: &std::collections::HashMap<String, String>) -> bool {
    attributes.contains_key("image-label")
        || attributes
            .get("ome")
            .and_then(|ome| serde_json::from_str::<serde_json::Value>(ome).ok())
            .is_some_and(|ome| ome.get("image-label").is_some())
}
