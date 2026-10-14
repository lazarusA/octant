//! Whether an opened array is a level of an OME-Zarr image, decided as
//! discovery decides it: the nearest ancestor group holding an image lists the
//! array's path among its levels.

use zarrs::array::DataType;

use super::parse::{is_ome_attributes, parse_group};
use super::reader::{MetaReader, join};
use super::variables::{draws_as_labels, level_dimension_names};

/// How the blocks of an OME-Zarr level array are read.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelArray {
    /// The image's axis names, as its variable has them.
    pub dim_names: Vec<String>,
    /// An integer label image: ids are read bit-exact.
    pub labels: bool,
}

/// The OME-Zarr level the array at `path` is, `None` for any other array
/// (including arrays under a non-OME group that has a `multiscales` list).
pub fn level_array(
    reader: &MetaReader<'_>,
    path: &str,
    rank: usize,
    data_type: &DataType,
) -> Option<LevelArray> {
    let path = path.trim_matches('/');
    for group in crate::utils::path::ancestor_paths(path) {
        let Ok(Some(attrs)) = reader.group_attributes(group) else {
            continue;
        };
        if !is_ome_attributes(&attrs) {
            continue;
        }
        let shapes = |p: &str| reader.array_shape(&join(group, p)).ok().flatten();
        let Some(parsed) = parse_group(std::sync::Arc::unwrap_or_clone(attrs), &shapes) else {
            continue;
        };
        let Some(multiscale) = parsed.fields.multiscales.as_ref().and_then(|m| m.first()) else {
            continue;
        };
        if multiscale.datasets.is_empty() {
            continue;
        }
        let relative = path[group.len()..].trim_matches('/');
        if !multiscale
            .datasets
            .iter()
            .any(|d| d.path.trim_matches('/') == relative)
        {
            return None;
        }
        let names = multiscale.axes.iter().map(|a| a.name.as_str());
        return Some(LevelArray {
            dim_names: level_dimension_names(names, rank),
            labels: draws_as_labels(parsed.fields.image_label.is_some(), data_type),
        });
    }
    None
}
