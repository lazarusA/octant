//! What OME-NGFF 0.1-0.3 metadata lacks compared to 0.4.
//!
//! 0.1 and 0.2 name no axes (the trailing `t, c, z, y, x` of the rank are
//! assumed), and none of them records `coordinateTransformations`: each
//! level's scale is the factor its space axes were downsampled by from the
//! first level. The rest (versions, string axes, the scales themselves) is
//! repaired by `repair.rs`, silently for these versions.

use serde_json::{Map, Value, json};

use super::axes::{AxisKind, default_axis_names, kind_from_name};
use super::parse::ShapeLookup;

/// Whether `version` predates 0.4.
pub fn is_legacy(version: &str) -> bool {
    matches!(version.trim(), "0.1" | "0.2" | "0.3")
}

/// Names the axes of 0.1-0.2 multiscales, which have none, after the rank of
/// their first level.
pub fn name_axes(ome: &mut Map<String, Value>, shapes: ShapeLookup<'_>) {
    let Some(list) = ome.get_mut("multiscales").and_then(Value::as_array_mut) else {
        return;
    };
    for multiscale in list.iter_mut().filter_map(Value::as_object_mut) {
        if multiscale
            .get("axes")
            .and_then(Value::as_array)
            .is_some_and(|a| !a.is_empty())
        {
            continue;
        }
        let rank = first_path(multiscale)
            .and_then(|p| shapes(&p))
            .map_or(0, |s| s.len());
        multiscale.insert("axes".into(), json!(default_axis_names(rank)));
    }
}

/// The path of the first dataset of `multiscale`, the base level.
fn first_path(multiscale: &Map<String, Value>) -> Option<String> {
    let first = multiscale.get("datasets")?.as_array()?.first()?;
    dataset_path(first.as_object()?)
}

fn dataset_path(dataset: &Map<String, Value>) -> Option<String> {
    let path = dataset.get("path")?.as_str()?;
    Some(path.trim_matches('/').to_string())
}

/// Gives each dataset without `coordinateTransformations` a scale: how much
/// its space axes were downsampled from the first level (1 on other axes and
/// when shapes are unknown). Returns whether any dataset changed.
pub fn fill_missing_scales(
    multiscale: &mut Map<String, Value>,
    axis_names: &[String],
    shapes: ShapeLookup<'_>,
) -> bool {
    let base = first_path(multiscale)
        .and_then(|p| shapes(&p))
        .unwrap_or_default();
    let Some(datasets) = multiscale.get_mut("datasets").and_then(Value::as_array_mut) else {
        return false;
    };
    let mut changed = false;
    // Each dataset with its own path: one without stays unscaled.
    for dataset in datasets.iter_mut().filter_map(Value::as_object_mut) {
        if dataset.contains_key("coordinateTransformations") {
            continue;
        }
        let shape = dataset_path(dataset)
            .and_then(|p| shapes(&p))
            .unwrap_or_default();
        let scale: Vec<f64> = axis_names
            .iter()
            .enumerate()
            .map(|(i, name)| match (base.get(i), shape.get(i)) {
                (Some(&b), Some(&s)) if s > 0 && kind_from_name(name) == AxisKind::Space => {
                    b as f64 / s as f64
                }
                _ => 1.0,
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
