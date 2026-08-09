# Octant Architecture Overview

Octant is a high-performance interactive visualization application for N-dimensional datasets (Zarr, Icechunk, NetCDF, etc.) built in Rust using [`eframe`/`egui`](https://github.com/emilk/egui) for the GUI and native [`wgpu`](https://github.com/gfx-rs/wgpu) for GPU rendering pipelines.

Repository: [https://github.com/lazarusA/octant](https://github.com/lazarusA/octant)

---

## 🏗 High-Level Architecture & Data Flow

```
                               ┌────────────────────────────────┐
                               │   Store / Dataset Selection    │
                               └───────────────┬────────────────┘
                                               │
                                               v
                               ┌────────────────────────────────┐
                               │        DatasetManager          │
                               │  (StoreHandle & DataSources)   │
                               └───────────────┬────────────────┘
                                               │
                                               v
┌───────────────────────────────┐     ┌─────────────────────────┐
│        BlockPrefetcher        │ ──► │       BlockCache        │
│   (Background Thread Pool)    │     │   (LRU Memory Cache)    │
└───────────────────────────────┘     └────────────┬────────────┘
                                                   │
                                                   v
                                      ┌─────────────────────────┐
                                      │       OctantBlock       │
                                      │ (Resident N-D Hyperslab)│
                                      └────────────┬────────────┘
                                                   │
                                                   v
                                      ┌─────────────────────────┐
                                      │       MatrixData        │
                                      │(2D/3D Renderable Payload)│
                                      └────────────┬────────────┘
                                                   │
                                                   v
                                      ┌─────────────────────────┐
                                      │      WGPU Renderers     │
                                      │(Matrix, Volume, Sphere) │
                                      └─────────────────────────┘
```

---

## 📁 Directory Structure & Key Modules

```
src/
├── app/                  # Application state, event loop, and data orchestration
├── data/                 # N-dimensional data system, caching, and backends
│   └── backends/         # Format-specific storage backends (Zarr, Icechunk)
├── plots/                # WGPU renderers, shaders, and 3D pipelines
├── ui/                   # egui GUI panels, overlays, and controls
├── utils/                # Grid orientation, coordinate discovery, and metadata
└── catalog/              # Pre-configured dataset catalog entries
```

---

### 1. [`src/app/`](https://github.com/lazarusA/octant/tree/main/src/app) — Application State & Orchestration

The `app` module manages main event loops, UI state, background tasks, and player controls.

- **[`state.rs`](https://github.com/lazarusA/octant/blob/main/src/app/state.rs)** & **[`mod.rs`](https://github.com/lazarusA/octant/blob/main/src/app/mod.rs)**: Defines `OctantApp`, holding global app state (selected store kind, target URI, active dataset metadata, dimension role configurations, colormaps, plot types, playback controls, `DatasetManager`, `BlockCache`, and `BlockPrefetcher`).
- **[`ui.rs`](https://github.com/lazarusA/octant/blob/main/src/app/ui.rs)**: Main `eframe::App::ui` entry point. Polling background prefetch results, timer animation loops, panel layouts, dynamic aspect ratio canvas allocation, and hover tooltip rendering.
- **[`data_loading.rs`](https://github.com/lazarusA/octant/blob/main/src/app/data_loading.rs)**: Non-blocking background metadata inspection (`inspect_active_store`).
- **[`block_loading.rs`](https://github.com/lazarusA/octant/blob/main/src/app/block_loading.rs)**: N-dimensional block loading (`load_selected_variable_block`), windowed hyperslab boundary calculations along animated dimensions, axis re-orientation via grid coordinates, and draining prefetcher results.
- **[`actions.rs`](https://github.com/lazarusA/octant/blob/main/src/app/actions.rs)**: `AppAction` event dispatch system for clean state mutation.

---

### 2. [`src/data/`](https://github.com/lazarusA/octant/tree/main/src/data) — Data System, Caching, and Backends

The `data` module provides a format-agnostic abstraction for loading, caching, and projecting N-dimensional hyperslabs into renderable payloads.

- **[`metadata.rs`](https://github.com/lazarusA/octant/blob/main/src/data/metadata.rs)**: Defines `DatasetMetadata` and `VariableInfo` for store inspection, variable discovery, shapes, dimensions, and `.zattrs` attributes.
- **[`octant_block.rs`](https://github.com/lazarusA/octant/blob/main/src/data/octant_block.rs)**: Resident in-memory representation of an N-dimensional block (`OctantBlock`).
  - Format-agnostic representation of arbitrary rank $N$.
  - Row-major stride indexing with fast element lookup (`get()`).
  - Projections: 2D slice (`slice_2d()`) into `MatrixData` and 3D volume (`volume()`).
- **[`block_store.rs`](https://github.com/lazarusA/octant/blob/main/src/data/block_store.rs)**: `BlockStore` trait defining unified backend capabilities (`backend_name`, `variables`, `inspect`, `fetch_block`, `fetch_blocks`).
- **[`store_handle.rs`](https://github.com/lazarusA/octant/blob/main/src/data/store_handle.rs)**: Thread-safe `StoreHandle` wrapping a `DataSource` and `Arc<dyn BlockStore>`.
- **[`dataset.rs`](https://github.com/lazarusA/octant/blob/main/src/data/dataset.rs)** & **[`dataset_manager.rs`](https://github.com/lazarusA/octant/blob/main/src/data/dataset_manager.rs)**: `DatasetManager` holds open `Dataset` instances keyed by unique `source_id`, preventing duplicate storage handle creation and preserving metadata for instant UI reactivation.
- **[`block_cache.rs`](https://github.com/lazarusA/octant/blob/main/src/data/block_cache.rs)**: LRU memory cache (`BlockCache`) for resident `OctantBlock` hyperslabs keyed by `BlockCacheKey`.
- **[`block_prefetch.rs`](https://github.com/lazarusA/octant/blob/main/src/data/block_prefetch.rs)**: Non-blocking background worker thread pool (`BlockPrefetcher`) for windowed lookahead prefetching along animated dimensions.
- **[`slice_request.rs`](https://github.com/lazarusA/octant/blob/main/src/data/slice_request.rs)** & **[`block_request.rs`](https://github.com/lazarusA/octant/blob/main/src/data/block_request.rs)**: Hyperslab selection specifications (`DimensionSelection::Range` vs `Index`) and block request batches.
- **[`matrix_data.rs`](https://github.com/lazarusA/octant/blob/main/src/data/matrix_data.rs)**: Standardized 2D/3D matrix data payload passed directly to GPU renderers.
- **[`source_factory.rs`](https://github.com/lazarusA/octant/blob/main/src/data/source_factory.rs)**: `SourceFactory::open(source)` initializing backend `StoreHandle` instances based on `DataSourceKind`.

#### Storage Backends ([`src/data/backends/`](https://github.com/lazarusA/octant/tree/main/src/data/backends))
- **[`backends/zarr.rs`](https://github.com/lazarusA/octant/blob/main/src/data/backends/zarr.rs)**: `ZarrBlockStore` implementing `BlockStore` for local Zarr directories and remote HTTP/S3 Zarr endpoints.
- **[`backends/icechunk.rs`](https://github.com/lazarusA/octant/blob/main/src/data/backends/icechunk.rs)**: `IcechunkBlockStore` implementing `BlockStore` for Icechunk transactional stores.
- **[`backends/zarr_block.rs`](https://github.com/lazarusA/octant/blob/main/src/data/backends/zarr_block.rs)**: Zarr array hyperslab extraction logic.
- **[`backends/zarr_storage.rs`](https://github.com/lazarusA/octant/blob/main/src/data/backends/zarr_storage.rs)** & **[`backends/icechunk_storage.rs`](https://github.com/lazarusA/octant/blob/main/src/data/backends/icechunk_storage.rs)**: Synchronous storage handle builders (`build_sync_store`, `open_local_storage`, `build_sync_icechunk_store`).

---

### 3. [`src/plots/`](https://github.com/lazarusA/octant/tree/main/src/plots) — WGPU Rendering Engine

Custom WGPU rendering pipelines for high-performance GPU visualization.

- **[`plot_type.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/plot_type.rs)**: `PlotType` enum (`Matrix`, `Line`, `Sphere`, `Surface`, `Volume`, `PointCloud`).
- **[`matrix.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/matrix.rs)**: 2D Heatmap matrix visualization with custom shaders, colormap sampling, NaN color masking, and clipping.
- **[`line.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/line.rs)**: 1D Line profile renderer.
- **[`sphere.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/sphere.rs)**: 3D Global spherical projection (equirectangular mapping with dynamic height displacement).
- **[`surface.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/surface.rs)**: 3D Mesh surface plot with elevation displacement.
- **[`volume.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/volume.rs)**: 3D Volumetric raymarching and isosurface extraction pipeline.
- **[`point_cloud.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/point_cloud.rs)**: 3D Point cloud scatter plot.

---

### 4. [`src/ui/`](https://github.com/lazarusA/octant/tree/main/src/ui) — GUI Overlays & Panels (`egui`)

Modular UI components integrated with `OctantApp`.

- **[`store.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/store.rs)**: Left collapsible panel. Store selection, URI input, active **Dataset Manager** list with instant dataset reactivation, and RAM cache statistics.
- **[`variables.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/variables.rs)**: Floating variable overlay listing variables in the active store.
- **[`variables_panel.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/variables_panel.rs)**: Controls panel for mapping dimensions to spatial roles ($X, Y, Z$) or Animation, double-slider hyperslab range selection, and variable metadata inspection.
- **[`top_bar.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/top_bar.rs)**: Header navigation bar with plot type selectors, colormap dropdowns, catalog overlay triggers, and cache settings.
- **[`bottom_bar.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/bottom_bar.rs)**: Animation playback controls, step sliders, timeline date bounds, and non-blocking status badges.
- **[`colorbar.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/colorbar.rs)**: Overlay displaying active colormaps, data ranges, NaN colors, and clipping bounds.
- **[`hover_tooltip.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/hover_tooltip.rs)**: Crosshair canvas reticle and mouse hover value inspector.
- **[`catalog.rs`](https://github.com/lazarusA/octant/blob/main/src/ui/catalog.rs)**: Sample dataset catalog overlay.

---

### 5. [`src/utils/`](https://github.com/lazarusA/octant/tree/main/src/utils) — Grid, Coordinates & Utilities

- **[`coordinates.rs`](https://github.com/lazarusA/octant/blob/main/src/utils/coordinates.rs)**: Rank-aware spatial coordinate candidate discovery (`get_cached_coord_bounds_with_rank`), searching latitude ($Y$, `rank - 2`) and longitude ($X$, `rank - 1`) coordinate arrays.
- **[`grid.rs`](https://github.com/lazarusA/octant/blob/main/src/utils/grid.rs)**: Grid orientation and axis flipping (`check_and_orient_axes_with_coords`) to ensure North-up and East-right spatial alignment.
- **[`metadata.rs`](https://github.com/lazarusA/octant/blob/main/src/utils/metadata.rs)**: Consolidated Zarr/Icechunk store variable and group attribute extractor.

---

## 🚀 Guidelines for Extending Octant

### Adding a New Storage Backend (e.g. NetCDF or GeoTIFF)
1. Create `src/data/backends/your_backend.rs`.
2. Implement the [`BlockStore`](https://github.com/lazarusA/octant/blob/main/src/data/block_store.rs) trait:
   - `backend_name(&self) -> &str`
   - `inspect(&self) -> Result<DatasetMetadata, BlockStoreError>`
   - `fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError>`
3. Add the new variant to `DataSourceKind` in [`data_source.rs`](https://github.com/lazarusA/octant/blob/main/src/data/data_source.rs) and wire it in [`SourceFactory::open`](https://github.com/lazarusA/octant/blob/main/src/data/source_factory.rs).

### Adding a New Plot Type or Renderer
1. Add a new enum variant to `PlotType` in [`plot_type.rs`](https://github.com/lazarusA/octant/blob/main/src/plots/plot_type.rs).
2. Create `src/plots/your_renderer.rs` implementing a WGPU rendering pipeline.
3. Instantiate the renderer in `OctantApp::new` and dispatch rendering in [`src/app/ui.rs`](https://github.com/lazarusA/octant/blob/main/src/app/ui.rs).

---

## 🔮 Future Architectural Roadmap: Multi-Variable Plotting

- **Multi-Layer Rendering Pipeline**: The strict separation between transient exploration state (`active_dataset_metadata`, `dim_config`) and active plotted state (`plotted_dataset_metadata`, `plotted_dim_config`, `plotted_selected_dim_ranges`, etc.) is designed to easily expand into a `Vec<PlottedVariableState>` or multi-layer pipeline.
- **Dimensional Compatibility Verification**: Variables across the same or different datasets with matching spatial ranks, shape dimensions, or spatial grid coordinates can be validated for dimensional compatibility and combined into:
  - Vector field overlays (e.g., $u$ and $v$ wind/current velocity components).
  - Multi-channel RGB/false-color composite layers.
  - Dual-curve line plots and multi-variable volumetric renderings.
