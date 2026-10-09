//! Overlays request, receive and show their own blocks at their own step, leaving the
//! base layer alone, and follow the base layer's step.

use std::sync::Arc;

use zarrs::array::{ArrayBuilder, ArraySubset, DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::storage::store::MemoryStore;

use crate::app::layers::{Source, VariableSelection};
use crate::app::{AnimationRole, OctantApp, SpatialRole, StoreKind};
use crate::data::backends::zarr::GenericZarrBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::{BlockCacheKey, DataSource, Dataset, DatasetMetadata, SliceRequest, StoreHandle};

const TARGET: &str = "memory://overlay";

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

/// An app whose dataset manager holds an in-memory store: `elev(lat, lon)` (5x4) and
/// `t2m`, `sst(time, lat, lon)` (3x5x4, one step per chunk). Nothing is plotted.
fn memory_app() -> (OctantApp, DatasetMetadata) {
    let store = Arc::new(MemoryStore::new());
    write_array(&store, "elev", &["lat", "lon"], &[5, 4], &[5, 4]);
    for name in ["t2m", "sst"] {
        write_array(
            &store,
            name,
            &["time", "lat", "lon"],
            &[3, 5, 4],
            &[1, 5, 4],
        );
    }

    let zarr = GenericZarrBlockStore::new(store, TARGET, "zarr", "Zarr");
    let meta = zarr.inspect().expect("inspect");
    let mut app = OctantApp::default();
    let source_id = StoreKind::make_source_id(StoreKind::RemoteZarr, TARGET);
    let kind = StoreKind::RemoteZarr.to_data_source_kind();
    let source = DataSource::new(&source_id, kind, TARGET, "Store");
    let handle = StoreHandle::new(source.clone(), Arc::new(zarr));
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    (app, meta)
}

/// `name`'s selection with default dimension roles, animated along `time` when it has one.
fn selection_of(app: &mut OctantApp, meta: &DatasetMetadata, name: &str) -> VariableSelection {
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
fn make_resident(app: &mut OctantApp, request: &SliceRequest) {
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
fn poll_until(app: &mut OctantApp, done: impl Fn(&OctantApp) -> bool) {
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

/// Asserts that the base layer requested and shows nothing.
fn assert_base_untouched(app: &OctantApp) {
    let base = &app.layers.base;
    assert!(base.data.matrix.is_none(), "the base layer shows nothing");
    assert!(base.load.slice_request.is_none());
    assert!(base.load.block_key.is_none());
}

#[test]
fn a_resident_overlay_block_is_shown_in_the_overlay_only() {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "elev");
    let id = app.layers.push(Source::Variable(selection));
    let request = app.staged_layer_request(id).expect("overlay request");
    make_resident(&mut app, &request.request);

    app.load_layer_block(id);

    let overlay = app.layers.get(id).expect("overlay");
    assert!(overlay.data.matrix.is_some(), "the overlay shows its block");
    assert_eq!(overlay.load.pending_target_step, None);
    let shown = overlay
        .load
        .slice_request
        .as_ref()
        .map(|r| r.variable.as_str());
    assert_eq!(shown, Some(request.var.name.as_str()));
    assert_base_untouched(&app);
}

#[test]
fn a_fetched_overlay_block_is_routed_to_the_overlay() {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "elev");
    let id = app.layers.push(Source::Variable(selection));

    app.load_layer_block(id);
    let overlay = app.layers.get(id).expect("overlay");
    assert!(
        overlay.load.block_key.is_some(),
        "the overlay requests its block"
    );
    assert_eq!(overlay.load.pending_target_step, Some(0));
    assert_base_untouched(&app);

    poll_until(&mut app, |app| {
        app.layers.get(id).is_some_and(|l| l.data.matrix.is_some())
    });
    let overlay = app.layers.get(id).expect("overlay");
    assert!(overlay.load.block_key.is_none(), "the request is settled");
    assert_base_untouched(&app);
}

#[test]
fn an_overlay_past_its_extent_shows_its_last_step() {
    let (mut app, meta) = memory_app();
    let selection = selection_of(&mut app, &meta, "t2m");
    let id = app.layers.push(Source::Variable(selection));
    make_resident(&mut app, &SliceRequest::full_range("t2m", &[3, 5, 4]));
    // The base layer's longer animated dimension is at step 7.
    app.current_timestep = 7;

    app.load_layer_block(id);

    assert_eq!(
        app.current_timestep, 7,
        "an overlay never moves the shared step"
    );
    assert_eq!(app.layer_step(id), 2);
    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_indices.first(), Some(&2));
    assert!(
        overlay.data.matrix.is_some(),
        "the clamped step is projected"
    );
}

#[test]
fn a_base_step_that_arrives_reloads_the_animated_overlays() {
    let (mut app, meta) = memory_app();
    let base = selection_of(&mut app, &meta, "t2m");
    let overlay = selection_of(&mut app, &meta, "sst");
    *app.layers.base.selection_mut() = base.clone();
    app.selected = base;
    let id = app.layers.push(Source::Variable(overlay));
    // Every step of the overlay is resident; the base layer's are fetched.
    make_resident(&mut app, &SliceRequest::full_range("sst", &[3, 5, 4]));

    app.request_step_or_load(1);
    assert_eq!(app.current_timestep, 0, "the base step is still loading");
    poll_until(&mut app, |app| app.current_timestep == 1);

    let overlay = app.layers.get(id).expect("overlay");
    assert_eq!(overlay.selection().dim_indices.first(), Some(&1));
    assert!(
        overlay.data.matrix.is_some(),
        "the overlay shows the new step"
    );
}
