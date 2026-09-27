//! Data structures and serde deserializers for OME-NGFF and OMERO metadata schemas.

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

pub(crate) fn de_opt_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
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

/// Root `.zattrs` representation containing OME-NGFF multiscales and OMERO rendering specs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RootZattrs {
    #[serde(default)]
    pub multiscales: Vec<Multiscale>,
    pub omero: Option<Omero>,
}

/// Multiscale image pyramid specification in OME-NGFF.
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

/// Named dimension axis in an OME-NGFF multiscale array.
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

/// Dataset level entry within an OME multiscale hierarchy.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiscaleDataset {
    pub path: String,
    #[serde(default)]
    #[serde(rename = "coordinateTransformations")]
    pub coordinate_transformations: Vec<CoordTransform>,
}

/// Coordinate transform (scale or translation) in OME-NGFF.
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

/// OMERO visualization and channel configuration root.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Omero {
    #[serde(default)]
    pub channels: Vec<OmeroChannel>,
    #[serde(default)]
    pub rdefs: Option<OmeroRdefs>,
}

/// Individual channel metadata in OMERO specifications.
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

/// Display intensity normalization window for an OMERO channel.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OmeroWindow {
    #[serde(default)]
    pub min: Option<f32>,
    #[serde(default)]
    pub max: Option<f32>,
    pub start: f32,
    pub end: f32,
}

/// Default rendering definitions (default plane coordinates and color model).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OmeroRdefs {
    #[serde(default, rename = "defaultT")]
    pub default_t: Option<usize>,
    #[serde(default, rename = "defaultZ")]
    pub default_z: Option<usize>,
    pub model: Option<String>,
}
