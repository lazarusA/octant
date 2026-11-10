//! Zarr block retrieval, slicing, and coordinate binding routines.

use std::collections::HashMap;
use std::sync::Arc;
use zarrs::array::chunk_cache::ChunkCacheDecodedLruSizeLimit;

use super::block_coords::{BlockWindow, StoreSource, block_coordinates};
use super::quantization::Quantization;
use super::slice::{ChunkReader, retrieve_array_subset_as_f32};
use super::storage::ZarrArrayHandle;
use crate::data::blocks::BlockStoreError;
use crate::data::coordinates::affine::GridAffine;
use crate::data::octant_block::OctantBlock;
use crate::data::ome::MetaReader;
use crate::data::ome::level::LevelArray;
use crate::data::slice_request::{SliceRequest, window_subset};
use crate::utils::metadata::attr_string;
use crate::utils::metadata::cf::is_spatial_key;
use zarrs::array::ArrayShardedReadableExtCache;

/// OME-Zarr group metadata, read by `ome::level` from the groups themselves
/// and large (a plate lists every well): not copied into blocks.
const OME_GROUP_KEYS: [&str; 8] = [
    "ome",
    "multiscales",
    "omero",
    "labels",
    "image-label",
    "plate",
    "well",
    "bioformats2raw.layout",
];

/// The array's attributes as text, plus those of its ancestor groups (e.g. DGGS
/// conventions, not OME-Zarr metadata) when the array has no `dggs` attribute
/// of its own.
pub(super) fn inherited_attributes(
    array: &ZarrArrayHandle,
    reader: &MetaReader<'_>,
    variable: &str,
) -> HashMap<String, String> {
    let mut attributes: HashMap<String, String> = array
        .attributes()
        .iter()
        .map(|(k, v)| (k.clone(), attr_string(v)))
        .collect();
    if attributes.contains_key("dggs") {
        return attributes;
    }
    for ancestor in crate::utils::path::ancestor_paths(variable) {
        // Groups the web fetched without `.zgroup` still have their attributes.
        let Ok(Some(group_attrs)) = reader.group_attributes(ancestor) else {
            continue;
        };
        let own_affine = attributes.contains_key("spatial:transform");
        for (k, v) in group_attrs.iter().filter(|(k, _)| {
            !OME_GROUP_KEYS.contains(&k.as_str()) && !(own_affine && is_spatial_key(k))
        }) {
            attributes
                .entry(k.clone())
                .or_insert_with(|| attr_string(v));
        }
        if attributes.contains_key("dggs") {
            break;
        }
    }
    attributes
}

/// The values of the window `origin..origin + shape` of `array`, through
/// `reader`; label ids kept bit-exact when `labels` (see
/// `retrieve_array_subset_as_label_bits`).
pub(super) fn read_window(
    array: &ZarrArrayHandle,
    reader: ChunkReader<'_>,
    variable: &str,
    (origin, shape): (&[usize], &[usize]),
    labels: bool,
) -> Result<Vec<f32>, BlockStoreError> {
    let subset = window_subset(origin, shape);
    log::debug!(
        "[ZarrBlock] Fetching '{variable}' subset {:?} (elements = {})",
        subset.to_ranges(),
        subset.num_elements(),
    );
    if labels
        && let Some(ids) =
            super::slice::retrieve_array_subset_as_label_bits(array, reader, &subset)?
    {
        return Ok(ids);
    }
    let values = retrieve_array_subset_as_f32(array, reader, &subset).map_err(|e| {
        log::error!("[ZarrBlock] Failed to retrieve array subset for '{variable}': {e:?}");
        BlockStoreError::from(e.to_string())
    })?;
    Ok(values)
}

/// How an opened array is read: through a cache of decoded chunks, or for
/// a sharded array, by the inner chunks a window touches (a decoded chunk
/// would be a whole shard).
pub(super) enum ArrayReader {
    Cached(Arc<ChunkCacheDecodedLruSizeLimit>),
    Sharded(ArrayShardedReadableExtCache),
}

impl ArrayReader {
    pub(super) fn chunk_reader(&self) -> ChunkReader<'_> {
        match self {
            Self::Cached(cache) => ChunkReader::Cached(cache),
            Self::Sharded(cache) => ChunkReader::Sharded(cache),
        }
    }
}

/// Pre-resolved metadata state for a cached Zarr array.
pub(super) struct ArrayCachedState {
    pub array: Arc<ZarrArrayHandle>,
    pub reader: ArrayReader,
    pub attributes: Arc<HashMap<String, String>>,
    /// Set for OME-Zarr level arrays, whose blocks keep their stored order.
    pub ome: Option<LevelArray>,
    /// Set for quantised embedding arrays, whose codes become values.
    pub quantization: Option<Quantization>,
    /// Set for arrays a `spatial:transform` places, whose x/y coordinates
    /// come from it rather than from coordinate arrays.
    pub affine: Option<GridAffine>,
}

/// Fetches a hyperslab of an opened array through its chunk cache, scaled by `scales` when quantised.
pub(super) fn fetch_block_from_cached_array(
    cached: &ArrayCachedState,
    scales: Option<&ArrayCachedState>,
    source: StoreSource<'_>,
    request: &SliceRequest,
) -> Result<OctantBlock, BlockStoreError> {
    let (origin, block_shape) = request.window(cached.array.shape())?;
    let mut raw_values = read_window(
        &cached.array,
        cached.reader.chunk_reader(),
        &request.variable,
        (&origin, &block_shape),
        cached.ome.as_ref().is_some_and(|l| l.labels),
    )?;
    if let Some(q) = &cached.quantization {
        q.apply(scales, (&origin, &block_shape), &mut raw_values)?;
    }
    if let Some(level) = &cached.ome {
        return OctantBlock::new(
            request.variable.clone(),
            block_shape,
            level.dim_names.clone(),
            origin,
            raw_values,
            HashMap::new(),
            cached.attributes.clone(),
        );
    }
    orient_and_build_block(cached, source, request, (origin, block_shape), raw_values)
}

/// The block of `raw_values` at the window `(origin, shape)`, with its
/// coordinates read from `source`, oriented north-up and west-left.
fn orient_and_build_block(
    cached: &ArrayCachedState,
    source: StoreSource<'_>,
    request: &SliceRequest,
    (origin, block_shape): (Vec<usize>, Vec<usize>),
    raw_values: Vec<f32>,
) -> Result<OctantBlock, BlockStoreError> {
    let full_shape = cached.array.shape();
    let dim_names = crate::utils::resolve_array_dimension_names(&cached.array);
    let window = BlockWindow {
        dim_names: &dim_names,
        full_shape,
        origin: &origin,
        block_shape: &block_shape,
        selections: &request.selections,
    };
    let placed = cached
        .affine
        .map(|a| a.coordinates(&dim_names, full_shape))
        .unwrap_or_default();
    let coords = block_coordinates(source, &request.variable, &window, &placed);
    coords.oriented_block(
        request.variable.clone(),
        raw_values,
        (dim_names, origin, block_shape),
        cached.attributes.clone(),
    )
}
