//! Upgrades OME-NGFF 0.1-0.3 metadata to 0.4, which `ome_zarr_metadata` reads.
//!
//! 0.1 and 0.2 name no axes (the trailing `t, c, z, y, x` of the rank are
//! assumed), 0.3 names them as plain strings, and none of them records
//! `coordinateTransformations`: each level's scale is the factor its space
//! axes were downsampled by from the first level.

use serde_json::{Map, Value, json};

use super::axes::{AxisKind, default_axis_names, kind_from_name};
use super::parse::ShapeLookup;

/// Whether `version` predates 0.4.
pub fn is_legacy(version: &str) -> bool {
    matches!(version.trim(), "0.1" | "0.2" | "0.3")
}

/// Rewrites 0.1-0.3 metadata in place as 0.4.
pub fn upgrade(ome: &mut Map<String, Value>, shapes: ShapeLookup<'_>) {
    if let Some(list) = ome.get_mut("multiscales").and_then(Value::as_array_mut) {
        for multiscale in list.iter_mut().filter_map(Value::as_object_mut) {
            upgrade_multiscale(multiscale, shapes);
        }
    }
    for key in ["image-label", "plate", "well"] {
        if let Some(obj) = ome.get_mut(key).and_then(Value::as_object_mut) {
            obj.insert("version".into(), json!("0.4"));
        }
    }
    fill_window_bounds(ome);
}

/// Early OMERO windows may omit `min` and `max`, which 0.4 requires: they
/// default to the window's own `start` and `end`.
fn fill_window_bounds(ome: &mut Map<String, Value>) {
    let channels = ome
        .get_mut("omero")
        .and_then(|o| o.get_mut("channels"))
        .and_then(Value::as_array_mut);
    for window in channels
        .into_iter()
        .flatten()
        .filter_map(|c| c.get_mut("window"))
        .filter_map(Value::as_object_mut)
    {
        for (bound, from) in [("min", "start"), ("max", "end")] {
            if !window.contains_key(bound)
                && let Some(v) = window.get(from).cloned()
            {
                window.insert(bound.into(), v);
            }
        }
    }
}

fn upgrade_multiscale(multiscale: &mut Map<String, Value>, shapes: ShapeLookup<'_>) {
    let paths = dataset_paths(multiscale);
    let rank = paths
        .first()
        .and_then(|p| shapes(p))
        .map(|s| s.len())
        .or_else(|| {
            multiscale
                .get("axes")
                .and_then(Value::as_array)
                .map(Vec::len)
        })
        .unwrap_or(0);
    let names: Vec<String> = match multiscale.get("axes").and_then(Value::as_array) {
        Some(axes) if !axes.is_empty() => axes.iter().map(axis_name).collect(),
        _ => default_axis_names(rank),
    };
    let axes: Vec<Value> = names.iter().map(|n| axis_object(n)).collect();
    multiscale.insert("axes".into(), Value::Array(axes));
    multiscale.insert("version".into(), json!("0.4"));
    fill_missing_scales(multiscale, &names, shapes);
}

fn axis_name(axis: &Value) -> String {
    match axis {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("dim")
            .to_string(),
        _ => "dim".to_string(),
    }
}

fn axis_object(name: &str) -> Value {
    match kind_from_name(name) {
        AxisKind::Space => json!({ "name": name, "type": "space" }),
        AxisKind::Time => json!({ "name": name, "type": "time" }),
        AxisKind::Channel => json!({ "name": name, "type": "channel" }),
        AxisKind::Other => json!({ "name": name }),
    }
}

fn dataset_paths(multiscale: &Map<String, Value>) -> Vec<String> {
    multiscale
        .get("datasets")
        .and_then(Value::as_array)
        .map(|ds| {
            ds.iter()
                .filter_map(|d| d.get("path").and_then(Value::as_str))
                .map(|p| p.trim_matches('/').to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Gives each dataset without `coordinateTransformations` a scale: how much
/// its space axes were downsampled from the first level (1 on other axes and
/// when shapes are unknown). Returns whether any dataset changed.
pub fn fill_missing_scales(
    multiscale: &mut Map<String, Value>,
    axis_names: &[String],
    shapes: ShapeLookup<'_>,
) -> bool {
    let paths = dataset_paths(multiscale);
    let base = paths.first().and_then(|p| shapes(p));
    let Some(datasets) = multiscale.get_mut("datasets").and_then(Value::as_array_mut) else {
        return false;
    };
    let mut changed = false;
    for (dataset, path) in datasets
        .iter_mut()
        .filter_map(Value::as_object_mut)
        .zip(&paths)
    {
        if dataset.contains_key("coordinateTransformations") {
            continue;
        }
        let shape = shapes(path);
        let scale: Vec<f64> = axis_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let downsampled = match (&base, &shape) {
                    (Some(b), Some(s)) => match (b.get(i), s.get(i)) {
                        (Some(&b), Some(&s)) if s > 0 => b as f64 / s as f64,
                        _ => 1.0,
                    },
                    _ => 1.0,
                };
                if kind_from_name(name) == AxisKind::Space {
                    downsampled
                } else {
                    1.0
                }
            })
            .collect();
        dataset.insert(
            "coordinateTransformations".into(),
            json!([{ "type": "scale", "scale": scale }]),
        );
        changed = true;
    }
    changed
}
