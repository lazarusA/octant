//! One `VariableInfo` per resolution level of an OME-Zarr image.

use std::collections::HashMap;
use std::sync::Arc;

use super::axes::default_axis_names;
use super::model::{OmeImage, OmeRef};
use super::reader::{MetaReader, Pending, join};
use crate::data::metadata::VariableInfo;
use crate::utils::metadata::{open_or_instantiate_array_normalized, variable_info_from_array};

/// The variables of `image`'s levels, named `{group}/{level path}`. Levels
/// whose array cannot be opened are left out; no array is built while any
/// level's metadata is still being fetched.
pub(super) fn image_variables(
    reader: &MetaReader<'_>,
    image: &Arc<OmeImage>,
) -> Result<Vec<VariableInfo>, Pending> {
    let mut found = Vec::with_capacity(image.levels.len());
    for (level, info) in image.levels.iter().enumerate() {
        let path = join(&image.group, &info.path);
        if let Some(meta) = reader.array_metadata(&path)? {
            found.push((level, path, meta));
        }
    }
    Ok(found
        .into_iter()
        .filter_map(|(level, path, meta)| level_variable(reader, image, level, &path, &meta))
        .collect())
}

/// The level array at `path`, built from its metadata document `meta`
/// (already read: no further request), else opened from the store.
fn level_variable(
    reader: &MetaReader<'_>,
    image: &Arc<OmeImage>,
    level: usize,
    path: &str,
    meta: &serde_json::Value,
) -> Option<VariableInfo> {
    let abs = format!("/{path}");
    let built = serde_json::from_value::<zarrs::metadata::ArrayMetadata>(
        crate::utils::metadata::normalize_v3_array_metadata(meta.clone()),
    )
    .ok()
    .and_then(|m| zarrs::array::Array::new_with_metadata(reader.storage(), &abs, m).ok());
    let array = match built {
        Some(array) => array,
        None => open_or_instantiate_array_normalized(reader.storage(), &abs).ok()?,
    };
    let info = variable_info_from_array(&array, path)?;
    let names = image.axes.iter().map(|a| a.name.as_str());
    Some(VariableInfo {
        dimension_names: level_dimension_names(names, info.shape.len()),
        long_name: image.name.clone(),
        attributes: HashMap::new(),
        ome: Some(OmeRef {
            image: image.clone(),
            level,
            labels: draws_as_labels(image.label_info.is_some(), array.data_type()),
        }),
        ..info
    })
}

/// The dimension names of a level array of rank `rank`: the image's axis
/// names, else the default `t, c, z, y, x` tail.
pub(super) fn level_dimension_names<'n>(
    names: impl ExactSizeIterator<Item = &'n str>,
    rank: usize,
) -> Vec<String> {
    if names.len() == rank {
        names.map(str::to_string).collect()
    } else {
        default_axis_names(rank)
    }
}

/// Whether a level draws as labels: only integer label images do (their ids
/// are read exactly).
pub(super) fn draws_as_labels(image_label: bool, dt: &zarrs::array::DataType) -> bool {
    use zarrs::array::data_type::*;
    image_label
        && (dt.is::<UInt8DataType>()
            || dt.is::<UInt16DataType>()
            || dt.is::<UInt32DataType>()
            || dt.is::<UInt64DataType>()
            || dt.is::<Int8DataType>()
            || dt.is::<Int16DataType>()
            || dt.is::<Int32DataType>()
            || dt.is::<Int64DataType>())
}
