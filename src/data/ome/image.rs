//! Builds `OmeImage` from a group's parsed `multiscales`, `omero` and
//! `image-label` fields.

use std::sync::Arc;

use ome_zarr_metadata::v0_5::{
    CoordinateTransform, CoordinateTransformScale, CoordinateTransformTranslation, ImageLabel,
    MultiscaleImage,
};

use super::axes::OmeAxis;
use super::model::{OmeImage, OmeLabelInfo, OmeLevel};
use super::omero;
use super::parse::ParsedGroup;

/// The image a parsed group describes, at store path `group` (`""` for the
/// root); `None` when the group has no usable multiscale.
pub fn image_from_group(group: &str, parsed: &ParsedGroup) -> Option<OmeImage> {
    let multiscale = parsed.fields.multiscales.as_ref()?.first()?;
    if multiscale.datasets.is_empty() {
        return None;
    }
    let axes: Vec<OmeAxis> = multiscale.axes.iter().map(OmeAxis::from_spec).collect();
    let rank = axes.len();
    let levels = multiscale
        .datasets
        .iter()
        .map(|d| level(multiscale, &d.path, &d.coordinate_transformations, rank))
        .collect();
    let defaults = omero::render_defaults(parsed.omero.as_ref());
    Some(OmeImage {
        group: group.trim_matches('/').to_string(),
        name: multiscale
            .name
            .clone()
            .or_else(|| omero_name(parsed.omero.as_ref()))
            .filter(|n| !n.trim().is_empty()),
        axes,
        levels,
        channels: omero::channels(parsed.omero.as_ref()),
        default_t: defaults.default_t,
        default_z: defaults.default_z,
        greyscale: defaults.greyscale,
        labels: Vec::new(),
        label_info: parsed
            .fields
            .image_label
            .as_ref()
            .map(|l| Arc::new(label_info(l))),
    })
}

/// The image name OMERO records (`omero.name`).
fn omero_name(omero: Option<&serde_json::Value>) -> Option<String> {
    omero?.get("name")?.as_str().map(|n| n.trim().to_string())
}

/// A level's scale and translation: its own transforms, then the multiscale's.
fn level(
    multiscale: &MultiscaleImage,
    path: &str,
    transforms: &[CoordinateTransform],
    rank: usize,
) -> OmeLevel {
    let mut scale = vec![1.0f64; rank];
    let mut translation = vec![0.0f64; rank];
    let shared = multiscale
        .coordinate_transformations
        .as_deref()
        .unwrap_or(&[]);
    for t in transforms.iter().chain(shared) {
        apply(t, &mut scale, &mut translation);
    }
    OmeLevel {
        path: path.trim_matches('/').to_string(),
        scale,
        translation,
    }
}

/// Composes `t` after the current mapping `x = translation + i * scale`.
/// Transforms stored in binary arrays (`path`) are not read and act as identity.
fn apply(t: &CoordinateTransform, scale: &mut [f64], translation: &mut [f64]) {
    match t {
        CoordinateTransform::Scale(CoordinateTransformScale::List { scale: s }) => {
            for (i, &f) in s.iter().enumerate().take(scale.len()) {
                let f = f64::from(f);
                if f.is_finite() && f != 0.0 {
                    scale[i] *= f;
                    translation[i] *= f;
                }
            }
        }
        CoordinateTransform::Translation(CoordinateTransformTranslation::List {
            translation: t,
        }) => {
            for (i, &d) in t.iter().enumerate().take(translation.len()) {
                if d.is_finite() {
                    translation[i] += f64::from(d);
                }
            }
        }
        _ => {}
    }
}

fn label_info(label: &ImageLabel) -> OmeLabelInfo {
    let mut colors: Vec<(u64, [u8; 4])> = label
        .colors
        .iter()
        .flatten()
        .map(|c| (c.label_value, c.rgba))
        .collect();
    colors.sort_by_key(|(v, _)| *v);
    colors.dedup_by_key(|(v, _)| *v);
    let mut properties_text: Vec<(u64, String)> = label
        .properties
        .iter()
        .flatten()
        .map(|p| {
            let fields: Vec<String> = p
                .properties
                .iter()
                .map(|(k, v)| format!("{k}: {}", property_text(v)))
                .collect();
            (p.label_value, fields.join(", "))
        })
        .collect();
    properties_text.sort_by_key(|(v, _)| *v);
    properties_text.dedup_by_key(|(v, _)| *v);
    OmeLabelInfo {
        colors,
        properties_text,
    }
}

fn property_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
