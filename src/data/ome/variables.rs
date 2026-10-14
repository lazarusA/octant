//! One `VariableInfo` per resolution level of an OME-Zarr image.

use std::collections::HashMap;
use std::sync::Arc;

use super::axes::default_axis_names;
use super::model::{OmeImage, OmeRef};
use super::reader::{MetaReader, Pending, join};
use crate::data::metadata::VariableInfo;
use crate::utils::metadata::{open_or_instantiate_array_normalized, variable_info_from_array};

/// The variables of `image`'s levels, named `{group}/{level path}`. Levels
/// whose array cannot be opened are left out.
pub fn image_variables(
    reader: &MetaReader<'_>,
    image: &Arc<OmeImage>,
    version: &str,
) -> Result<Vec<VariableInfo>, Pending> {
    let mut variables = Vec::with_capacity(image.levels.len());
    let mut waiting = false;
    for (level, info) in image.levels.iter().enumerate() {
        let path = join(&image.group, &info.path);
        match reader.array_metadata(&path) {
            Err(Pending) => waiting = true,
            Ok(None) => {}
            Ok(Some(meta)) => {
                if let Some(var) = level_variable(reader, image, level, &path, meta, version) {
                    variables.push(var);
                }
            }
        }
    }
    if waiting {
        return Err(Pending);
    }
    Ok(variables)
}

/// The level array at `path`, built from its metadata document `meta`
/// (already read: no further request), else opened from the store.
fn level_variable(
    reader: &MetaReader<'_>,
    image: &Arc<OmeImage>,
    level: usize,
    path: &str,
    meta: serde_json::Value,
    version: &str,
) -> Option<VariableInfo> {
    let abs = format!("/{path}");
    let built = serde_json::from_value::<zarrs::metadata::ArrayMetadata>(
        crate::utils::metadata::normalize_v3_array_metadata(meta),
    )
    .ok()
    .and_then(|m| zarrs::array::Array::new_with_metadata(reader.storage(), &abs, m).ok());
    let array = match built {
        Some(array) => array,
        None => open_or_instantiate_array_normalized(reader.storage(), &abs).ok()?,
    };
    let mut attributes = HashMap::new();
    attributes.insert("ome_version".to_string(), version.to_string());
    // Only integer label images draw as labels (their ids are read exactly).
    if image.label_info.is_some() && is_integer(array.data_type()) {
        attributes.insert(LABEL_IMAGE_ATTRIBUTE.to_string(), "true".to_string());
    }
    let info = variable_info_from_array(&array, path)?;
    Some(VariableInfo {
        dimension_names: level_dimension_names(image, info.shape.len()),
        long_name: image.name.clone(),
        attributes,
        ome: Some(OmeRef {
            image: image.clone(),
            level,
        }),
        ..info
    })
}

/// Variable attribute of label images drawn as labels (integer ids).
pub const LABEL_IMAGE_ATTRIBUTE: &str = "ome_label_image";

/// The dimension names of an `image` level array of rank `rank`: the image's
/// axes, else the default `t, c, z, y, x` tail.
pub(super) fn level_dimension_names(image: &OmeImage, rank: usize) -> Vec<String> {
    if image.axes.len() == rank {
        image.axes.iter().map(|a| a.name.clone()).collect()
    } else {
        default_axis_names(rank)
    }
}

pub(super) fn is_integer(dt: &zarrs::array::DataType) -> bool {
    use zarrs::array::data_type::*;
    dt.is::<UInt8DataType>()
        || dt.is::<UInt16DataType>()
        || dt.is::<UInt32DataType>()
        || dt.is::<UInt64DataType>()
        || dt.is::<Int8DataType>()
        || dt.is::<Int16DataType>()
        || dt.is::<Int32DataType>()
        || dt.is::<Int64DataType>()
}
