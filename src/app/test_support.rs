//! Test fixtures: an app over an in-memory Zarr store, variable selections
//! of it, resident blocks, and polling for fetched ones.

use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;

use crate::app::layers::VariableSelection;
use crate::app::{AnimationRole, OctantApp, SpatialRole, StoreKind};
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{BlockCacheKey, DataSource, Dataset, DatasetMetadata, SliceRequest, StoreHandle};

pub(crate) const TARGET: &str = "memory://overlay";

/// Writes `name` (`dims`, `shape`, chunked by `chunks`) holding `0, 1, 2, ...`.
fn write_array(store: &Arc<MemoryStore>, name: &str, dims: &[&str], shape: &[u64], chunks: &[u64]) {
    let f32_dt = DataType::from_metadata(&MetadataV3::new("float32")).expect("float32");
    let mut builder = ArrayBuilder::new(
        shape.to_vec(),
        chunks.to_vec(),
        f32_dt,
        FillValue::from(0f32),
    );
    builder.dimension_names(Some(dims.to_vec()));
    let array = builder
        .build(store.clone(), &format!("/{name}"))
        .expect("build");
    array.store_metadata().expect("metadata");
    let len = shape.iter().product::<u64>() as usize;
    let values: Vec<f32> = (0..len).map(|i| i as f32).collect();
    let all = ArraySubset::new_with_shape(shape.to_vec());
    array.store_array_subset(&all, &values).expect("values");
}

/// An app whose dataset manager holds an in-memory store: `elev(lat, lon)` (5x4),
/// `t2m`, `sst(time, lat, lon)` (3x5x4, one step per chunk) and `rgb(band, lat,
/// lon)` (3x5x4). Nothing is plotted.
pub(crate) fn memory_app() -> (OctantApp, DatasetMetadata) {
    let mut app = OctantApp::default();
    let meta = add_memory_dataset(&mut app, TARGET);
    (app, meta)
}

/// Registers another in-memory store holding the same arrays (and no
/// coordinates) under `target`; returns its metadata.
pub(crate) fn add_memory_dataset(app: &mut OctantApp, target: &str) -> DatasetMetadata {
    let store = Arc::new(MemoryStore::new());
    write_array(&store, "elev", &["lat", "lon"], &[5, 4], &[5, 4]);
    write_array(
        &store,
        "rgb",
        &["band", "lat", "lon"],
        &[3, 5, 4],
        &[3, 5, 4],
    );
    for name in ["t2m", "sst"] {
        write_array(
            &store,
            name,
            &["time", "lat", "lon"],
            &[3, 5, 4],
            &[1, 5, 4],
        );
    }

    let zarr = GenericZarrBlockStore::new(store, target, "zarr", "Zarr");
    let meta = zarr.inspect().expect("inspect");
    let source_id = StoreKind::make_source_id(StoreKind::RemoteZarr, target);
    let kind = StoreKind::RemoteZarr.to_data_source_kind();
    let source = DataSource::new(&source_id, kind, target, "Store");
    let handle = StoreHandle::new(source.clone(), Arc::new(zarr));
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    meta
}

/// `name`'s selection with default dimension roles, animated along `time` when it has one.
pub(crate) fn selection_of(
    app: &mut OctantApp,
    meta: &DatasetMetadata,
    name: &str,
) -> VariableSelection {
    let idx = meta
        .variables
        .iter()
        .position(|v| v.name.trim_matches('/') == name)
        .expect("variable");
    let var = meta.variables.get(idx).cloned().expect("variable");
    crate::ui::variables_panel::init_variable_dimension_defaults(app, &var);
    let mut selection = VariableSelection {
        store_kind: StoreKind::RemoteZarr,
        store_target: TARGET.to_string(),
        metadata: Some(meta.clone()),
        variable_idx: idx,
        ..std::mem::take(&mut app.selected)
    };
    if let Some(time) = var.dimension_names.iter().position(|d| d == "time") {
        selection.dim_config[time].spatial = SpatialRole::None;
        selection.dim_config[time].animation = AnimationRole::Animated;
        selection.dim_config[time].active = true;
        selection.animated_dim = Some(time);
        selection.spatial_dims.retain(|&d| d != time);
    }
    selection
}

/// Puts `request`'s block of the in-memory store into the app's block cache.
pub(crate) fn make_resident(app: &mut OctantApp, request: &SliceRequest) {
    let source_id = StoreKind::make_source_id(StoreKind::RemoteZarr, TARGET);
    let handle = app
        .dataset_manager
        .get(&source_id)
        .expect("store")
        .store
        .clone();
    let block = handle.fetch(request).expect("block");
    app.block_cache
        .put(BlockCacheKey::new(source_id, request), block);
}

/// Polls finished fetches until `done` holds.
pub(crate) fn poll_until(app: &mut OctantApp, done: impl Fn(&OctantApp) -> bool) {
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(10);
    while !done(app) {
        assert!(
            web_time::Instant::now() < deadline,
            "the block never arrived"
        );
        app.poll_block_prefetch_results();
        std::thread::yield_now();
    }
}
