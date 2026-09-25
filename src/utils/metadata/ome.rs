//! OME-NGFF multiscales and OMERO channel metadata parsing.

use std::collections::HashMap;
use std::sync::Arc;

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use zarrs::array::Array;
use zarrs::storage::ReadableStorageTraits;

use crate::data::metadata::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;

fn de_opt_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    match v {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        Some(_) => Ok(None),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RootZattrs {
    #[serde(default)]
    pub multiscales: Vec<Multiscale>,
    pub omero: Option<Omero>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Multiscale {
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub axes: Vec<Axis>,
    #[serde(default)]
    pub datasets: Vec<MultiscaleDataset>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Axis {
    pub name: String,
    pub unit: Option<String>,
}

impl<'de> Deserialize<'de> for Axis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::String(s) => Ok(Axis {
                name: s,
                unit: None,
            }),
            serde_json::Value::Object(map) => {
                let name = map
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("dim")
                    .to_string();
                let unit = map
                    .get("unit")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());
                Ok(Axis { name, unit })
            }
            _ => Ok(Axis {
                name: "dim".to_string(),
                unit: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiscaleDataset {
    pub path: String,
    #[serde(default)]
    #[serde(rename = "coordinateTransformations")]
    pub coordinate_transformations: Vec<CoordTransform>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CoordTransform {
    #[serde(rename = "scale")]
    Scale { scale: Vec<f32> },
    #[serde(rename = "translation")]
    Translation { translation: Vec<f32> },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Omero {
    #[serde(default)]
    pub channels: Vec<OmeroChannel>,
    #[serde(default)]
    pub rdefs: Option<OmeroRdefs>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OmeroChannel {
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub label: Option<String>,
    pub name: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    pub window: Option<OmeroWindow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OmeroWindow {
    #[serde(default)]
    pub min: Option<f32>,
    #[serde(default)]
    pub max: Option<f32>,
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OmeroRdefs {
    #[serde(default, rename = "defaultT")]
    pub default_t: Option<usize>,
    #[serde(default, rename = "defaultZ")]
    pub default_z: Option<usize>,
    pub model: Option<String>,
}

/// Normalizes NGFF attributes across v0.1–v0.5 schemas into a standard root map.
pub fn normalize_ngff_attributes(
    attrs: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    if attrs.contains_key("multiscales") {
        return attrs;
    }
    let Some(ome) = attrs.get("ome").and_then(|v| v.as_object()).cloned() else {
        return attrs;
    };
    let mut out = ome;
    if !out.contains_key("omero")
        && let Some(omero) = attrs.get("omero").cloned()
    {
        out.insert("omero".to_string(), omero);
    }
    out
}

/// Extracts `VariableInfo` list for all pyramid levels in an OME-NGFF group.
pub fn extract_ome_multiscale_variables(
    store: Arc<dyn ReadableStorageTraits>,
    root_attrs_map: &serde_json::Map<String, serde_json::Value>,
    group_prefix: &str,
) -> Vec<VariableInfo> {
    let normalized = normalize_ngff_attributes(root_attrs_map.clone());
    let Ok(root_zattrs) =
        serde_json::from_value::<RootZattrs>(serde_json::Value::Object(normalized))
    else {
        return Vec::new();
    };

    let Some(multiscale) = root_zattrs.multiscales.first() else {
        return Vec::new();
    };

    let channel_labels: Vec<String> = root_zattrs
        .omero
        .as_ref()
        .map(|o| {
            o.channels
                .iter()
                .enumerate()
                .map(|(i, ch)| {
                    ch.name
                        .as_deref()
                        .or(ch.label.as_deref())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("Channel {i}"))
                })
                .collect()
        })
        .unwrap_or_default();

    let channel_colors: Vec<String> = root_zattrs
        .omero
        .as_ref()
        .map(|o| {
            o.channels
                .iter()
                .map(|ch| ch.color.clone().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();

    let channel_windows: Vec<String> = root_zattrs
        .omero
        .as_ref()
        .map(|o| {
            o.channels
                .iter()
                .map(|ch| {
                    if let Some(ref w) = ch.window {
                        format!("{}:{}", w.start, w.end)
                    } else {
                        String::new()
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let channel_actives: Vec<String> = root_zattrs
        .omero
        .as_ref()
        .map(|o| {
            o.channels
                .iter()
                .map(|ch| ch.active.map(|a| a.to_string()).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();

    let axes_names: Vec<String> = multiscale.axes.iter().map(|a| a.name.clone()).collect();
    let mut variables = Vec::new();

    for ds in &multiscale.datasets {
        let clean_path = ds.path.trim_matches('/');
        let var_name = if group_prefix.is_empty() {
            clean_path.to_string()
        } else {
            format!("{}/{}", group_prefix.trim_matches('/'), clean_path)
        };

        let zarr_path = format!("/{var_name}");
        let array_opt = Array::open(store.clone(), &zarr_path)
            .ok()
            .or_else(|| Array::open(store.clone(), &format!("/{clean_path}")).ok())
            .or_else(|| {
                crate::utils::metadata::open_or_instantiate_array_normalized(
                    store.clone(),
                    &zarr_path,
                )
                .ok()
            });

        if let Some(arr) = array_opt {
            let shape = arr.shape().to_vec();
            if shape.is_empty() {
                continue;
            }
            let zero_idx = vec![0u64; shape.len()];
            let chunk_shape = arr
                .chunk_shape(&zero_idx)
                .map(|v| v.into_iter().map(|n| n.get()).collect::<Vec<u64>>())
                .unwrap_or_else(|_| shape.clone());
            let data_type = format!("{:?}", arr.data_type());

            let dimension_names = if axes_names.len() == shape.len() {
                axes_names.clone()
            } else {
                fallback_ome_axes_for_rank(shape.len())
            };

            let mut attrs: HashMap<String, String> = HashMap::new();
            if !channel_labels.is_empty() {
                attrs.insert("omero_channels".to_string(), channel_labels.join(","));
            }
            if !channel_colors.is_empty() {
                attrs.insert("omero_colors".to_string(), channel_colors.join(","));
            }
            if !channel_windows.is_empty() {
                attrs.insert("omero_windows".to_string(), channel_windows.join(","));
            }
            if !channel_actives.is_empty() {
                attrs.insert("omero_actives".to_string(), channel_actives.join(","));
            }
            if let Some(ref omero) = root_zattrs.omero
                && let Some(ref rdefs) = omero.rdefs
            {
                if let Some(dz) = rdefs.default_z {
                    attrs.insert("default_z".to_string(), dz.to_string());
                }
                if let Some(dt) = rdefs.default_t {
                    attrs.insert("default_t".to_string(), dt.to_string());
                }
            }
            for trans in &ds.coordinate_transformations {
                if let CoordTransform::Scale { scale } = trans
                    && scale.len() == dimension_names.len()
                {
                    for (dim_name, &s_val) in dimension_names.iter().zip(scale.iter()) {
                        if s_val > 0.0 {
                            attrs.insert(format!("scale_{dim_name}"), s_val.to_string());
                        }
                    }
                }
            }

            let file_size = calculate_variable_size_bytes(&shape, &data_type);
            variables.push(VariableInfo {
                name: var_name,
                data_type,
                shape,
                dimension_names,
                chunk_shape,
                file_size,
                units: None,
                long_name: multiscale.name.clone(),
                time_coverage_start: None,
                time_coverage_end: None,
                temporal_resolution: None,
                attributes: attrs,
            });
        }
    }

    variables
}

/// Fallback OME axis names when axes are omitted in older NGFF specifications.
pub fn fallback_ome_axes_for_rank(rank: usize) -> Vec<String> {
    match rank {
        5 => vec![
            "t".to_string(),
            "c".to_string(),
            "z".to_string(),
            "y".to_string(),
            "x".to_string(),
        ],
        4 => vec![
            "c".to_string(),
            "z".to_string(),
            "y".to_string(),
            "x".to_string(),
        ],
        3 => vec!["c".to_string(), "y".to_string(), "x".to_string()],
        2 => vec!["y".to_string(), "x".to_string()],
        _ => (0..rank).map(|i| format!("dim_{i}")).collect(),
    }
}
