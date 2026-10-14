//! Repairs common deviations from the OME-Zarr schemas before parsing, so
//! data written by older or lenient tools still opens. Every repair is
//! recorded as a warning.

use serde_json::{Map, Value, json};

use super::axes::{AxisKind, kind_from_name};
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

/// Repairs metadata in place: namespaced (0.5) metadata keeps its version on
/// the `ome` object only, free metadata (0.4) on each object. What `legacy`
/// (0.1-0.3) metadata lacks by its own spec is filled in without a warning.
pub fn repair(
    ome: &mut Map<String, Value>,
    shapes: ShapeLookup<'_>,
    namespaced: bool,
    legacy: bool,
    warnings: &mut Vec<String>,
) {
    let mut fix = Fix {
        namespaced,
        legacy,
        warnings,
    };
    if let Some(list) = ome.get_mut("multiscales").and_then(Value::as_array_mut) {
        for (i, m) in list.iter_mut().filter_map(Value::as_object_mut).enumerate() {
            let at = format!("multiscales[{i}]");
            fix.version(m, &at);
            fix.multiscale(m, &at, shapes);
        }
    }
    for key in ["image-label", "plate", "well"] {
        if let Some(obj) = ome.get_mut(key).and_then(Value::as_object_mut) {
            fix.version(obj, key);
        }
    }
    repair_groups(ome, fix.warnings);
}

struct Fix<'w> {
    namespaced: bool,
    legacy: bool,
    warnings: &'w mut Vec<String>,
}

impl Fix<'_> {
    /// Records a deviation legacy metadata may have by its spec.
    fn legacy_warning(&mut self, warning: impl FnOnce() -> String) {
        if !self.legacy {
            self.warnings.push(warning());
        }
    }

    fn version(&mut self, obj: &mut Map<String, Value>, at: &str) {
        if self.namespaced {
            obj.remove("version");
            return;
        }
        match obj.get("version").and_then(Value::as_str) {
            Some("0.4") => return,
            Some(v) => self.legacy_warning(|| format!("{at}.version \"{v}\" read as 0.4")),
            None => {}
        }
        obj.insert("version".into(), json!("0.4"));
    }

    fn multiscale(&mut self, m: &mut Map<String, Value>, at: &str, shapes: ShapeLookup<'_>) {
        retain_keys(m, MULTISCALE_KEYS, at, self.warnings);
        let mut names = Vec::new();
        for (j, axis) in m
            .get_mut("axes")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
            .enumerate()
        {
            if let Value::String(name) = axis {
                self.legacy_warning(|| format!("{at}.axes: \"{name}\" is not an axis object"));
                *axis = axis_object(name);
            }
            if let Some(obj) = axis.as_object_mut() {
                // Some writers spell the unit `units`.
                if !obj.contains_key("unit")
                    && let Some(units) = obj.remove("units")
                {
                    let w = format!("{at}.axes[{j}]: \"units\" read as \"unit\"");
                    self.warnings.push(w);
                    obj.insert("unit".into(), units);
                }
                retain_keys(obj, AXIS_KEYS, &format!("{at}.axes[{j}]"), self.warnings);
            }
            let name = axis.get("name").and_then(Value::as_str).unwrap_or("");
            names.push(name.to_string());
        }
        let datasets = m.get_mut("datasets").and_then(Value::as_array_mut);
        for (j, d) in datasets
            .into_iter()
            .flatten()
            .filter_map(Value::as_object_mut)
            .enumerate()
        {
            let at = format!("{at}.datasets[{j}]");
            retain_keys(d, DATASET_KEYS, &at, self.warnings);
        }
        if fill_missing_scales(m, &names, shapes) {
            self.legacy_warning(|| format!("{at}.datasets: missing coordinateTransformations"));
        }
    }
}

/// A typed axis named `name`, its type guessed from the name.
fn axis_object(name: &str) -> Value {
    match kind_from_name(name) {
        AxisKind::Space => json!({ "name": name, "type": "space" }),
        AxisKind::Time => json!({ "name": name, "type": "time" }),
        AxisKind::Channel => json!({ "name": name, "type": "channel" }),
        AxisKind::Other => json!({ "name": name }),
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
