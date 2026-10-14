//! Group attributes -> OME-Zarr fields, through `ome_zarr_metadata`.
//!
//! Metadata of every version is read as 0.5 fields: 0.1-0.3 get the axis names
//! they lack (`legacy`), known quirks are repaired (`repair`), and each field is
//! parsed on its own so one bad field does not hide the others. Whatever had to
//! be changed, dropped or failed validation is kept in `warnings`.

use ome_zarr_metadata::{Validate, v0_4, v0_5};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use super::{legacy, repair};

/// Shapes of the arrays at the given store paths, when known (for metadata
/// that predates `coordinateTransformations`).
pub type ShapeLookup<'a> = &'a dyn Fn(&str) -> Option<Vec<u64>>;

/// One group's OME-Zarr metadata.
#[derive(Debug, Clone, Default)]
pub struct ParsedGroup {
    /// The version the metadata declares (`"0.4"`).
    pub version: String,
    pub fields: v0_5::OmeFields,
    /// The raw `omero` object, read channel by channel (`omero.rs`).
    pub omero: Option<Value>,
    /// What did not conform, in English (logged; one toast per dataset).
    pub warnings: Vec<String>,
}

/// Whether a group's attributes hold OME-Zarr metadata: an `ome` object,
/// or OME fields of the right shape (a `labels` list of names or an `omero`
/// object alone, which other conventions use too, need a `multiscales`).
pub fn is_ome_attributes(attrs: &Map<String, Value>) -> bool {
    if attrs.get("ome").is_some_and(Value::is_object) {
        return true;
    }
    let array = |k: &str| attrs.get(k).is_some_and(Value::is_array);
    let object = |k: &str| attrs.get(k).is_some_and(Value::is_object);
    let label_names = attrs
        .get("labels")
        .and_then(Value::as_array)
        .is_some_and(|l| !l.is_empty() && l.iter().all(Value::is_string));
    array("multiscales")
        || object("plate")
        || object("well")
        || object("image-label")
        || attrs.contains_key("bioformats2raw.layout")
        || (label_names && attrs.len() <= 2)
}

/// Parses a group's attributes; `None` when they are not OME-Zarr.
pub fn parse_group(mut attrs: Map<String, Value>, shapes: ShapeLookup<'_>) -> Option<ParsedGroup> {
    if !is_ome_attributes(&attrs) {
        return None;
    }
    // 0.5 keeps its fields in `ome`; some writers leave `omero` beside it.
    let namespaced = attrs.get("ome").is_some_and(Value::is_object);
    let (mut ome, outer_omero) = match attrs.remove("ome") {
        Some(Value::Object(ome)) => (ome, attrs.remove("omero")),
        other => {
            if let Some(other) = other {
                attrs.insert("ome".into(), other);
            }
            (attrs, None)
        }
    };
    let mut warnings = Vec::new();
    let version = declared_version(&ome, namespaced);
    let old = !namespaced && legacy::is_legacy(&version);
    if old {
        legacy::name_axes(&mut ome, shapes);
    }
    if namespaced && version != "0.5" {
        warnings.push(format!("version {version} read as 0.5"));
    }
    repair::repair(&mut ome, shapes, namespaced, old, &mut warnings);
    let fields = fields(&ome, namespaced, &mut warnings);
    if let Err(e) = fields.validate() {
        warnings.push(e.to_string());
    }
    Some(ParsedGroup {
        version,
        fields,
        omero: ome.remove("omero").or(outer_omero),
        warnings,
    })
}

/// The version a group declares; unversioned free metadata is 0.4, or 0.2
/// when its multiscales name no axes.
fn declared_version(ome: &Map<String, Value>, namespaced: bool) -> String {
    if namespaced {
        return ome
            .get("version")
            .and_then(json_text)
            .unwrap_or_else(|| "0.5".into());
    }
    let first_multiscale = ome
        .get("multiscales")
        .and_then(Value::as_array)
        .and_then(|m| m.first());
    // Unversioned multiscales without axes are 0.1/0.2 (axes came with 0.3).
    let unversioned = if first_multiscale.is_some_and(|m| m.get("axes").is_none()) {
        "0.2"
    } else {
        "0.4"
    };
    first_multiscale
        .and_then(|m| m.get("version"))
        .or_else(|| {
            ["image-label", "plate", "well"]
                .iter()
                .find_map(|k| ome.get(*k)?.get("version"))
        })
        .and_then(json_text)
        .unwrap_or_else(|| unversioned.into())
}

/// A JSON string (trimmed, non-empty) or number, as text.
pub(super) fn json_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn field<T: DeserializeOwned>(value: &Value, what: &str, warnings: &mut Vec<String>) -> Option<T> {
    T::deserialize(value)
        .map_err(|e| warnings.push(format!("{what}: {e}")))
        .ok()
}

/// Each `multiscales` entry parsed on its own; entries that fail are dropped.
fn multiscales<T: DeserializeOwned>(
    ome: &Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Option<Vec<T>> {
    let list = ome.get("multiscales")?.as_array()?;
    let parsed: Vec<T> = list
        .iter()
        .enumerate()
        .filter_map(|(i, m)| field(m, &format!("multiscales[{i}]"), warnings))
        .collect();
    (!parsed.is_empty()).then_some(parsed)
}

fn bioformats2raw(
    ome: &Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Option<v0_5::Bioformats2Raw> {
    let layout = ome.get("bioformats2raw.layout")?;
    let mut map = Map::new();
    map.insert("bioformats2raw.layout".into(), layout.clone());
    if let Some(series) = ome.get("series") {
        map.insert("series".into(), series.clone());
    }
    field(&Value::Object(map), "bioformats2raw.layout", warnings)
}

/// The fields as 0.5 types, 0.4 metadata read as 0.4 and converted. The
/// `omero` field is left out: channels are read from the raw JSON.
fn fields(ome: &Map<String, Value>, namespaced: bool, w: &mut Vec<String>) -> v0_5::OmeFields {
    macro_rules! parse {
        ($key:literal) => {
            ome.get($key).and_then(|v| field(v, $key, w))
        };
    }
    if namespaced {
        v0_5::OmeFields {
            version: Default::default(),
            bioformats2raw: bioformats2raw(ome, w),
            multiscales: multiscales(ome, w),
            labels: parse!("labels"),
            image_label: parse!("image-label"),
            plate: parse!("plate"),
            well: parse!("well"),
            omero: None,
        }
    } else {
        v0_4::OmeNgffGroupAttributes {
            bioformats2raw: bioformats2raw(ome, w),
            multiscales: multiscales(ome, w),
            labels: parse!("labels"),
            image_label: parse!("image-label"),
            plate: parse!("plate"),
            well: parse!("well"),
            omero: None,
        }
        .into()
    }
}
