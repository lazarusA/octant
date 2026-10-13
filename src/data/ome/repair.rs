//! Repairs common deviations from the OME-Zarr schemas before parsing, so
//! data written by older or lenient tools still opens. Every repair is
//! recorded as a warning.

use serde_json::{Map, Value, json};

use super::legacy::fill_missing_scales;
use super::parse::ShapeLookup;

const MULTISCALE_KEYS: &[&str] = &[
    "version",
    "name",
    "axes",
    "datasets",
    "coordinateTransformations",
    "type",
    "metadata",
];
const DATASET_KEYS: &[&str] = &["path", "coordinateTransformations"];
const AXIS_KEYS: &[&str] = &["name", "type", "unit"];
const IMAGE_LABEL_KEYS: &[&str] = &["version", "colors", "properties", "source"];
const PLATE_KEYS: &[&str] = &[
    "version",
    "acquisitions",
    "columns",
    "field_count",
    "name",
    "rows",
    "wells",
];
const ACQUISITION_KEYS: &[&str] = &[
    "id",
    "name",
    "maximumfieldcount",
    "description",
    "starttime",
    "endtime",
];
const PLATE_WELL_KEYS: &[&str] = &["path", "rowIndex", "columnIndex"];
const WELL_KEYS: &[&str] = &["version", "images"];
const WELL_IMAGE_KEYS: &[&str] = &["path", "acquisition"];

/// Repairs free (0.4) metadata in place.
pub fn repair_v04(
    ome: &mut Map<String, Value>,
    shapes: ShapeLookup<'_>,
    warnings: &mut Vec<String>,
) {
    if let Some(list) = ome.get_mut("multiscales").and_then(Value::as_array_mut) {
        for (i, m) in list.iter_mut().filter_map(Value::as_object_mut).enumerate() {
            let at = format!("multiscales[{i}]");
            set_version(m, "0.4", &at, warnings);
            repair_multiscale(m, &at, shapes, warnings);
        }
    }
    for key in ["image-label", "plate", "well"] {
        if let Some(obj) = ome.get_mut(key).and_then(Value::as_object_mut) {
            set_version(obj, "0.4", key, warnings);
        }
    }
    repair_groups(ome, warnings);
}

/// Repairs namespaced (0.5) metadata in place: 0.5 keeps the version on the
/// `ome` object only.
pub fn repair_v05(
    ome: &mut Map<String, Value>,
    shapes: ShapeLookup<'_>,
    warnings: &mut Vec<String>,
) {
    if let Some(list) = ome.get_mut("multiscales").and_then(Value::as_array_mut) {
        for (i, m) in list.iter_mut().filter_map(Value::as_object_mut).enumerate() {
            let at = format!("multiscales[{i}]");
            m.remove("version");
            repair_multiscale(m, &at, shapes, warnings);
        }
    }
    for key in ["image-label", "plate", "well"] {
        if let Some(obj) = ome.get_mut(key).and_then(Value::as_object_mut) {
            obj.remove("version");
        }
    }
    repair_groups(ome, warnings);
}

fn repair_multiscale(
    m: &mut Map<String, Value>,
    at: &str,
    shapes: ShapeLookup<'_>,
    warnings: &mut Vec<String>,
) {
    retain_keys(m, MULTISCALE_KEYS, at, warnings);
    let mut names = Vec::new();
    if let Some(axes) = m.get_mut("axes").and_then(Value::as_array_mut) {
        for (j, axis) in axes.iter_mut().enumerate() {
            if let Value::String(name) = axis {
                warnings.push(format!("{at}.axes: \"{name}\" is not an axis object"));
                *axis = json!({ "name": name.clone() });
            }
            if let Some(obj) = axis.as_object_mut() {
                // Some writers spell the unit `units`.
                if !obj.contains_key("unit")
                    && let Some(units) = obj.remove("units")
                {
                    warnings.push(format!("{at}.axes[{j}]: \"units\" read as \"unit\""));
                    obj.insert("unit".into(), units);
                }
                retain_keys(obj, AXIS_KEYS, &format!("{at}.axes[{j}]"), warnings);
            }
            names.push(
                axis.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            );
        }
    }
    if let Some(datasets) = m.get_mut("datasets").and_then(Value::as_array_mut) {
        for (j, d) in datasets
            .iter_mut()
            .filter_map(Value::as_object_mut)
            .enumerate()
        {
            retain_keys(d, DATASET_KEYS, &format!("{at}.datasets[{j}]"), warnings);
        }
    }
    if fill_missing_scales(m, &names, shapes) {
        warnings.push(format!("{at}.datasets: missing coordinateTransformations"));
    }
}

fn repair_groups(ome: &mut Map<String, Value>, warnings: &mut Vec<String>) {
    if let Some(label) = ome.get_mut("image-label").and_then(Value::as_object_mut) {
        retain_keys(label, IMAGE_LABEL_KEYS, "image-label", warnings);
    }
    if let Some(plate) = ome.get_mut("plate").and_then(Value::as_object_mut) {
        retain_keys(plate, PLATE_KEYS, "plate", warnings);
        retain_in_list(plate, "acquisitions", ACQUISITION_KEYS, warnings);
        retain_in_list(plate, "wells", PLATE_WELL_KEYS, warnings);
        retain_in_list(plate, "rows", &["name"], warnings);
        retain_in_list(plate, "columns", &["name"], warnings);
    }
    if let Some(well) = ome.get_mut("well").and_then(Value::as_object_mut) {
        retain_keys(well, WELL_KEYS, "well", warnings);
        retain_in_list(well, "images", WELL_IMAGE_KEYS, warnings);
    }
    let channels = ome
        .get_mut("omero")
        .and_then(|o| o.get_mut("channels"))
        .and_then(Value::as_array_mut);
    for channel in channels.into_iter().flatten() {
        if let Some(Value::String(color)) = channel.get_mut("color")
            && let Some(hex) = color.strip_prefix('#')
        {
            *color = hex.to_string();
        }
    }
}

fn set_version(obj: &mut Map<String, Value>, version: &str, at: &str, warnings: &mut Vec<String>) {
    let current = obj.get("version").and_then(Value::as_str);
    if current != Some(version) {
        if let Some(v) = current {
            warnings.push(format!("{at}.version \"{v}\" read as {version}"));
        }
        obj.insert("version".into(), json!(version));
    }
}

fn retain_keys(
    obj: &mut Map<String, Value>,
    allowed: &[&str],
    at: &str,
    warnings: &mut Vec<String>,
) {
    let unknown: Vec<String> = obj
        .keys()
        .filter(|k| !allowed.contains(&k.as_str()))
        .cloned()
        .collect();
    for key in unknown {
        warnings.push(format!("{at}: unknown key \"{key}\""));
        obj.remove(&key);
    }
}

fn retain_in_list(
    obj: &mut Map<String, Value>,
    list: &str,
    allowed: &[&str],
    warnings: &mut Vec<String>,
) {
    if let Some(items) = obj.get_mut(list).and_then(Value::as_array_mut) {
        for (i, item) in items
            .iter_mut()
            .filter_map(Value::as_object_mut)
            .enumerate()
        {
            retain_keys(item, allowed, &format!("{list}[{i}]"), warnings);
        }
    }
}
