//! Whether an opened array is a level of an OME-Zarr image, decided as
//! discovery decides it: the nearest ancestor group holding an image lists the
//! array's path among its levels.

use zarrs::array::DataType;

use super::image::image_from_group;
use super::parse::parse_group;
use super::reader::{MetaReader, join};
use super::variables::{is_integer, level_dimension_names};

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
        let shapes = |p: &str| reader.array_shape(&join(group, p)).ok().flatten();
        let Some(image) = parse_group(&attrs, &shapes).and_then(|g| image_from_group(group, &g))
        else {
            continue;
        };
        let relative = path[group.len()..].trim_matches('/');
        if !image.levels.iter().any(|l| l.path == relative) {
            return None;
        }
        return Some(LevelArray {
            dim_names: level_dimension_names(&image, rank),
            labels: image.label_info.is_some() && is_integer(data_type),
        });
    }
    None
}
