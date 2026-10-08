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

## 🔮 Roadmap: Layers, Overlays, Operations & Multi-Scale (TODO)

### Where we stand

- **Done (behavior unchanged):** plotted state lives in a `LayerStack` ([`src/app/layers/`](https://github.com/lazarusA/octant/tree/main/src/app/layers)). The UI's staged selection is `OctantApp::selected` (a `VariableSelection`); Plot copies it to the base layer, read through `OctantApp::plotted()`. Each `Layer` has a read-only `LayerId` given by the stack (`LayerId::BASE` for the base layer) and owns its `Source` (only `Source::Variable` so far), `LayerData`, `LayerRenderers`, `ColorStyle`, `CompositeStyle` and `LoadState`. Painting, OIT release, volume uploads and pyramid tile resampling loop over `LayerStack::iter`/`iter_mut`, and `ColorStyle::params` builds the shader color uniforms. `block_axes` places a selection inside a block, and `VariableSelection::slice_request` builds requests. All of these are pure or work per layer.
- **Done: projection and paint per layer.** `apply_block_projection`, `apply_2d_projection`/`apply_3d_volume_projection`, `commit_volume_slab` and both pipeline rebuilds take a `LayerId` and write only that layer (`LayerStack::get`/`get_mut`). Coastline meshes and the line plot settings follow the base layer only. They read the layer's selection through `layer_selections` (`None` once the layer is gone): the base layer falls back to the staged `selected` until its first plot, as the `effective_*` helpers (now `layer_*(LayerId::BASE)`) always did, and other layers stage their own. `poll_block_prefetch_results` sends a requested block to the layer whose request it is (`find_by_key` returns its id) and any other block to the layer whose view it belongs to (`layer_showing_block`). `get_color_params`, `transparency_mode` and the mesh, volume and point cloud uniform getters take the `&Layer` they draw; the colormap picker's preview applies to the base layer only (`layer_colormap`). Volume data extents go through `ColorStyle::reset_to_extent`/`follow_extent` (which skip non-finite ends; the 2D rebuild still writes its extent inline, NaN included), and dirty volume planes through `LayerRenderers::mark_volume_dirty`.
- **Still tied to the base layer** (the next work of the overlay phase):
  - Requests: `load_selected_variable_block`, the cache-hit path (`show_cached_block`, `project_cached_volume_blocks`), prefetching and playback start from the staged selection and serve the base layer.
  - Hover, colorbar, axis labels, aspect ratios and volume shifts, and export read the base layer only (the view frame is the base layer's).
  - `LayerStack::push` adds an overlay under a new `LayerId` (the stack hands out every id; `LayerId` has no `Default`), and every per-layer loop already includes overlays, but nothing pushes one yet. When the UI does, put the id in egui salts and the per-plot caches (hover, composite labels) next to `metadata_generation`.
- **Decisions taken:** the plot type is part of the selection, and the canvas type is the base layer's. Colormap reversal stays in `ColormapState`; move it into `ColorStyle` when overlays need their own. Settings for each plot type (`sphere_mode`, `volume_*`, `line_*`) stay on the app until overlays of those plot types exist.

### Model to grow into

```rust
enum Source {
    Variable(VariableSelection),                              // exists
    Channel  { source, dim, index },                          // u/v or bands of one variable
    Derived  { expr: Expr, inputs: Vec<SourceId>, target },   // var3 = f(var1, var2)
    Reduce   { source, dim, op },                             // time mean, anomaly base
    Mosaic   { members },                                     // nested global + regional datasets
}
// Each layer then gets a style: Scalar (ColorStyle), Rgb (CompositeStyle),
// Vectors { u, v, glyph, color_by }, Bivariate { x, y, lut, ranges }
```

Sources produce arrays aligned to a grid; layers draw one or more sources in a style. Every source variant carries a `VariableSelection` for the dimensions it is shown on.

### TODO, in order

1. **Same-grid overlays.**
   - Push more layers onto `LayerStack`, routed by `LayerId` and `find_by_key`, and request their blocks (see "Still tied to the base layer").
   - Draw overlays after the base and before coastlines, with their own `ColorStyle`, opacity and visibility. NaN must draw transparent.
   - UI:
     - "Add as overlay" on variable rows.
     - A layer list in the docked panel: visible, opacity, colormap, reorder, remove (`ui.close_button`).
     - Stacked colorbars.
     - One hover section per layer.
   - Classify compatibility (`SameGrid`, `Geo { bbox }`, `IndexOnly`, `Incompatible`) in a new alignment module. This replaces the deleted shape-equality check.
2. **Overlays across datasets (regional on global).**
   - Place an overlay with the heatmap's `tile_bounds`: the overlay's lon/lat bounding box normalized into the base layer's lon/lat frame, as coastlines already do with `dataset_geo_bounds`.
   - Normalize longitude conventions (0–360 vs −180–180), and split quads that cross the antimeridian.
   - The pyramid resampler also writes `tile_bounds`, so combine the two, or give overlays no pyramid at first.
   - Match time and level by coordinate value (nearest within a tolerance, using `CoordValues`), not by index. Show a "nearest" note when they differ.
   - Allow regular and 1D-irregular overlays first. Curvilinear and HEALPix work only as the base until the shader handles them.
   - The view frame is the base layer's; a "fit all layers" option can come later.
   - Sphere and surface overlays: a second mesh renderer using its own lon/lat bounds (`has_reference_globe`), a small radial offset, and OIT.
3. **Operations (`Source::Derived`).**
   - Compute on the CPU from each step's projected 2D slices; the result is a `MatrixData` that every renderer already draws.
   - Same grid: element by element. Mixed grids: regrid onto a chosen target grid (nearest or bilinear over lon/lat).
   - Cache results in an LRU keyed by the input block keys, the expression's hash and each source's `metadata_generation`.
   - Write our own small AST, evaluated one whole array per node, with NaN propagating (+ − × ÷, comparisons, where/mask, abs, sqrt, log, hypot, atan2, clamp). Start with a fixed menu of operations, then free-form text.
   - Derive units for the simple cases, warn when + or − mixes units, and reject cycles.
   - `Source::Reduce` (time mean, anomalies) loads the animated range; it comes later.
4. **Bivariate maps.** Look each cell up in a 2D color table on the CPU and draw the result through the existing RGB composite path (`COLORMAP_RGB_COMPOSITE`). Its legend is a 2D square. A GPU version (two data buffers plus a dedicated 2D lookup texture, not more colormap-atlas rows) can follow.
5. **Vector fields (u, v).**
   - Magnitude and direction need nothing new: they are derived sources (`hypot`, `atan2` with a cyclic colormap).
   - Arrows: a new `VectorRenderer` that instances arrows on the GPU and pulls `u`/`v` from storage buffers (as the AGENTS.md rules require for grid data), sampled at a stride that follows the zoom.
   - Streamlines: CPU line meshes, which the rules allow.
   - Mind screen-y vs north, block flips (`flipped_dims`), tangent frames on the sphere, and grid-relative components on rotated or curvilinear grids.
   - Dense textures and animated particles come last.
6. **Multi-scale exploration.**
   - Give `Source::Variable` an optional `ScaleLevels` ladder:
     - OME-NGFF multiscales: today each level is listed as its own variable; group them instead.
     - GeoTIFF overviews: today each overview is listed as its own variable; group them instead.
     - Zarr and Icechunk multiscale groups.
     - Keep `MatrixPyramid` as the fallback for data already in memory.
   - Widen `slice_request` into a view request (visible bounding box, pixels, step). It picks the coarsest level with at least one cell per pixel and a chunk-aligned window.
   - Refine progressively: a coarse backdrop for the whole area plus a fine viewport tile, both layers of the same source.
   - Request only after the view settles (`view_interacting`). Play back at the coarse level and refine when paused.
   - Base the color range on a stable reference (the coarsest level's or `valid_min`/`valid_max`), not on the current tile.
   - Label operations computed from coarser data with their level, and offer an "exact" mode.
   - Categorical data needs levels built with the mode, not the mean.
   - Then add `Source::Mosaic` for nested datasets.
7. **Later:** volume overlays (same grid only, as texture channels), line plots with several series and a second y-axis, and saving and restoring sessions (serialize sources and layers).
