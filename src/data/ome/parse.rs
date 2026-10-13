//! Group attributes -> OME-Zarr fields, through `ome_zarr_metadata`.
//!
//! Metadata of every version is read as 0.5 fields: 0.1-0.3 are first upgraded
//! to 0.4 (`legacy`), known quirks are repaired (`repair`), and each field is
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

/// The multiscale level paths named by a group, before parsing (to read their
/// shapes first).
pub fn level_paths(attrs: &Map<String, Value>) -> Vec<String> {
    let ome = attrs.get("ome").and_then(Value::as_object).unwrap_or(attrs);
    ome.get("multiscales")
        .and_then(Value::as_array)
        .and_then(|ms| ms.first())
        .and_then(|m| m.get("datasets"))
        .and_then(Value::as_array)
        .map(|ds| {
            ds.iter()
                .filter_map(|d| d.get("path").and_then(Value::as_str))
                .map(|p| p.trim_matches('/').to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Parses a group's attributes; `None` when they are not OME-Zarr.
pub fn parse_group(attrs: &Map<String, Value>, shapes: ShapeLookup<'_>) -> Option<ParsedGroup> {
    if !is_ome_attributes(attrs) {
        return None;
    }
    let namespaced = attrs.get("ome").and_then(Value::as_object);
    let mut ome = namespaced.cloned().unwrap_or_else(|| attrs.clone());
    if namespaced.is_some()
        && !ome.contains_key("omero")
        && let Some(omero) = attrs.get("omero")
    {
        ome.insert("omero".to_string(), omero.clone());
    }

    let mut warnings = Vec::new();
    let version = declared_version(&ome, namespaced.is_some());
    if namespaced.is_none() && legacy::is_legacy(&version) {
        legacy::upgrade(&mut ome, shapes);
    }
    let fields = if namespaced.is_some() {
        if version != "0.5" {
            warnings.push(format!("version {version} read as 0.5"));
        }
        repair::repair_v05(&mut ome, shapes, &mut warnings);
        fields_v05(&ome, &mut warnings)
    } else {
        repair::repair_v04(&mut ome, shapes, &mut warnings);
        fields_v04(&ome, &mut warnings)
    };
    if let Err(e) = fields.validate() {
        warnings.push(e.to_string());
    }
    Some(ParsedGroup {
        version,
        fields,
        omero: ome.get("omero").cloned(),
        warnings,
    })
}

/// The version a group declares, `"0.4"` when unversioned metadata is free.
fn declared_version(ome: &Map<String, Value>, namespaced: bool) -> String {
    let as_text = |v: &Value| match v {
        Value::String(s) => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    };
    if namespaced {
        return ome
            .get("version")
            .and_then(as_text)
            .unwrap_or_else(|| "0.5".into());
    }
    let first_multiscale = ome
        .get("multiscales")
        .and_then(Value::as_array)
        .and_then(|m| m.first())
        .and_then(|m| m.get("version"));
    first_multiscale
        .or_else(|| {
            ["image-label", "plate", "well"]
                .iter()
                .find_map(|k| ome.get(*k)?.get("version"))
        })
        .and_then(as_text)
        .unwrap_or_else(|| "0.4".into())
}

fn field<T: DeserializeOwned>(value: &Value, what: &str, warnings: &mut Vec<String>) -> Option<T> {
    serde_json::from_value(value.clone())
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

fn fields_v04(ome: &Map<String, Value>, warnings: &mut Vec<String>) -> v0_5::OmeFields {
    let get = |k: &str| ome.get(k);
    let mut w = Vec::new();
    let attrs = v0_4::OmeNgffGroupAttributes {
        bioformats2raw: bioformats2raw(ome, &mut w),
        multiscales: multiscales::<v0_4::MultiscaleImage>(ome, &mut w),
        labels: get("labels").and_then(|v| field(v, "labels", &mut w)),
        image_label: get("image-label").and_then(|v| field(v, "image-label", &mut w)),
        plate: get("plate").and_then(|v| field(v, "plate", &mut w)),
        well: get("well").and_then(|v| field(v, "well", &mut w)),
        omero: get("omero").and_then(|v| field(v, "omero", &mut w)),
    };
    warnings.extend(w);
    attrs.into()
}

fn fields_v05(ome: &Map<String, Value>, warnings: &mut Vec<String>) -> v0_5::OmeFields {
    let get = |k: &str| ome.get(k);
    let mut w = Vec::new();
    let fields = v0_5::OmeFields {
        version: Default::default(),
        bioformats2raw: bioformats2raw(ome, &mut w),
        multiscales: multiscales::<v0_5::MultiscaleImage>(ome, &mut w),
        labels: get("labels").and_then(|v| field(v, "labels", &mut w)),
        image_label: get("image-label").and_then(|v| field(v, "image-label", &mut w)),
        plate: get("plate").and_then(|v| field(v, "plate", &mut w)),
        well: get("well").and_then(|v| field(v, "well", &mut w)),
        omero: get("omero").and_then(|v| field(v, "omero", &mut w)),
    };
    warnings.extend(w);
    fields
}
