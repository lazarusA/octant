//! One `VariableInfo` per resolution level of an OME-Zarr image.

use std::sync::Arc;

use super::model::{OmeImage, OmeRef};
use super::reader::{MetaReader, Pending, join};
use crate::data::metadata::VariableInfo;
use crate::utils::metadata::{open_or_instantiate_array_normalized, variable_info_from_array};

/// The variables of `image`'s levels, named `{group}/{level path}`. Levels
/// whose array cannot be opened are left out; no array is built while any
/// level's metadata is still being fetched, but every missing level is asked
/// for in the same pass.
pub(super) fn image_variables(
    reader: &MetaReader<'_>,
    image: &Arc<OmeImage>,
) -> Result<Vec<VariableInfo>, Pending> {
    let mut found = Vec::with_capacity(image.levels.len());
    let mut pending = false;
    for (level, info) in image.levels.iter().enumerate() {
        let path = join(&image.group, &info.path);
        match reader.array_metadata(&path) {
            Ok(Some(meta)) => found.push((level, path, meta)),
            Ok(None) => {}
            Err(Pending) => pending = true,
        }
    }
    if pending {
        return Err(Pending);
    }
    Ok(found
        .into_iter()
        .filter_map(|(level, path, meta)| level_variable(reader, image, level, &path, &meta))
        .collect())
}

/// The level array at `path`, built from its metadata document `meta`
/// (already read: no further request), else opened from the store. An array
/// whose rank differs from the image's axes stays a plain Zarr variable.
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
    let Some(dimension_names) = level_dimension_names(names, info.shape.len()) else {
        log::warn!(
            "OME-Zarr level '{path}' has {} dimensions but its image lists {} axes; read as plain Zarr",
            info.shape.len(),
            image.axes.len()
        );
        return Some(info);
    };
    Some(VariableInfo {
        dimension_names,
        long_name: info.long_name.or_else(|| image.name.clone()),
        ome: Some(OmeRef {
            image: image.clone(),
            level,
            labels: draws_as_labels(image.label_info.is_some(), array.data_type()),
        }),
        ..info
    })
}

/// The dimension names of a level array of rank `rank`: the image's axis
/// names, `None` when their count differs from the rank (the axes' units,
/// scales and kinds would then land on the wrong dimensions).
pub(super) fn level_dimension_names<'n>(
    names: impl ExactSizeIterator<Item = &'n str>,
    rank: usize,
) -> Option<Vec<String>> {
    (names.len() == rank).then(|| names.map(str::to_string).collect())
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
