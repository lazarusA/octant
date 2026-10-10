# Octant Architecture Overview

Octant is a high-performance interactive visualization application for N-dimensional datasets (Zarr, Icechunk, NetCDF and GeoTIFF) built in Rust (Rust 2024 edition) using [`eframe`/`egui`](https://github.com/emilk/egui) for the immediate-mode GUI and native [`wgpu`](https://github.com/gfx-rs/wgpu) for GPU rendering pipelines.

Repository: [https://github.com/lazarusA/octant](https://github.com/lazarusA/octant)

---

## 1. High-Level Architecture & End-to-End Data Flow

The diagram below presents the core subsystems of Octant and how data flows from storage formats into GPU shaders, canvas interaction, and presentation.

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                   Storage & Ingestion Subsystem (src/data/backends/)             │
│  ├── POSIX Local Files       ├── HTTP / S3 Range Requests                        │
│  ├── Zarr v2 / v3 Arrays     ├── Icechunk Snapshot Datasets                      │
│  ├── NetCDF-3 / NetCDF-4     └── GeoTIFF / Cloud-Optimized GeoTIFF (COG)         │
└────────────────────────────────────────┬─────────────────────────────────────────┘
                                         │ BlockStore trait (inspect, fetch_block, coords)
                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│                        N-D Data Engine Subsystem (src/data/)                     │
│  ├── BlockStore Trait        (inspect metadata, variable coordinates, fetch)     │
│  ├── CoordinateLoader        (Background async coordinate axis streaming)        │
│  ├── BlockPrefetcher         (Background thread pool for predictive fetch)      │
│  ├── BlockCache              (LRU in-memory cache of resident OctantBlocks)      │
│  ├── CoordBoundsCache        (Global coordinate domain bounds cache)             │
│  ├── OctantBlock             (Resident N-D tensor slice with Arc<[f32]> storage) │
│  └── Grid & Axis Orientation (Flips North-up, West-left, transposes lon/lat)     │
└────────────────────────────────────────┬─────────────────────────────────────────┘
                                         │ OctantBlock (Oriented resident tensor)
                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│             Layers & Slicing Subsystem (src/data/slicing/ & src/app/layers/)     │
│  ├── Slicing Routines        (1D profiles, 2D planar hyperslabs, 3D volumes)     │
│  ├── Compositing             (RGB / CMYK multi-channel composite blending)       │
│  ├── LayerStack              (Base Layer + multi-variable Overlay alignment)     │
│  ├── LayerData               (MatrixData, VolumeData, LineLayout)                │
│  └── ColorStyle              (Colormap row index, dynamic/locked range, opacity) │
└──────────────────┬─────────────────────────────────────────────┬─────────────────┘
                   │ LayerData (tensors & geometry)              │ ColorStyle & LUT
                   ▼                                             ▼
┌──────────────────────────────────────────────┐ ┌─────────────────────────────────┐
│     GPU Rendering Subsystem (src/plots/)     │ │ Shared Colormap Atlas @group(1) │
│  ├── Heatmap / Flatmap Matrix Pipeline       │ │ (256×K RGBA8 texture shared by  │
│  ├── 1D Multi-Series Line Plot Pipeline      │ │  all pipelines & CPU sampling)  │
│  ├── 3D Displaced Mesh (Surface & Sphere)    │ └────────────────┬────────────────┘
│  ├── 3D Raymarched Volume (DVR trilinear)    │                  │
│  ├── 3D Point Cloud Billboard Pipeline       │                  │
│  ├── Coastline Vector Boundary Overlays      │                  │
│  └── Order-Independent Transparency (OIT)    │◄─────────────────┘
└──────────────────────┬───────────────────────┘
                       │ Render callbacks & offscreen frames
                       ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│             Presentation & Canvas Subsystems (src/app/canvas/ & src/ui/)         │
│  ├── Canvas Engine           (src/app/canvas/: viewport, axes, gestures, overlay)│
│  ├── Plot Controllers        (src/app/controllers/: polymorphic plot navigation) │
│  ├── Immediate-Mode UI       (src/ui/: top bar, bottom timeline, panels, modals) │
│  ├── Hover Inspector         (src/ui/hover/: cell sampling, raycasts, reticle)   │
│  └── Figure Export           (src/export/: PNG, WebP P3, SVG, PDF, clipboard)    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Low-Level Pipeline Lifecycle & Execution Sequence

The breakdown below traces the exact types, asynchronous channels, LRU caches, memory representations, uniform layouts, and GPU passes that power Octant's frame cycle.

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        Detailed Frame Lifecycle & Data Flow                            │
└────────────────────────────────────────────────────────────────────────────────────────┘

Phase 1: Store Discovery & Async Inspection
│
├── 1. User submits path/URI or Drag-and-Drop file
│   └── SourceFactory::open(source) -> StoreHandle
├── 2. Background channel inspection (std::sync::mpsc)
│   └── BlockStore::inspect() -> DatasetMetadata (dimensions, variables, attributes)
└── 3. Background coordinate loading
    ├── CoordinateLoader::request_variable_coordinates(var)
    └── BlockStore::variable_coordinates() -> CoordValues (Regular, Values, Labels)
        └── coordinates_revision bumped on arrival

Phase 2: Hyperslab Request & LRU Block Cache
│
├── 1. Staged VariableSelection (roles, slice ranges, animation step)
├── 2. Cache key lookup: BlockCache::get(BlockKey)
│   ├── Cache Hit: Returns Arc<OctantBlock> immediately
│   └── Cache Miss:
│       ├── BlockPrefetcher::request_block(SliceRequest)
│       ├── Background worker reads BlockStore::fetch_block() -> OctantBlock
│       └── Inserted into LRU BlockCache (evicts least recently used blocks)
└── 3. Grid & Axis Orientation
    └── check_and_orient_block_grid(OctantBlock, OrientHints)
        ├── Transposes lon-first blocks (lon, lat) -> (lat, lon)
        └── Reverses coordinates for North-up and West-left display

Phase 3: Slicing & LayerStack Compositing
│
├── 1. Hyperslab extraction from oriented block:
│   ├── 2D Heatmap / Flatmap -> MatrixData (width, height, Arc<[f32]>, GridTopology)
│   ├── 3D Volume -> VolumeData (width, height, depth, validity mask)
│   └── 1D Line Plot -> LineLayout::lines() (strided multi-series sample buffer)
├── 2. LayerStack synchronization:
│   └── Syncs base layer & overlay layers (SameGrid, Geo, IndexOnly alignments)
└── 3. Color styling:
    └── ColorStyle resolves colormap atlas row, data bounds (min/max), and opacity curve

Phase 4: GPU Uniform Assembly & Render Execution
│
├── 1. Colormap Atlas update:
│   └── colormap_atlas::prepare() uploads dirty rows to shared 256×K RGBA8 texture
├── 2. Viewport calculation (src/app/canvas/viewport.rs):
│   └── Computes aspect scaling, pan/zoom uniforms, and screen-space plot rect
├── 3. Render pass execution (src/plots/):
│   ├── 2D Heatmap / 1D Line: Direct WGPU paint callback with transformed uniforms
│   ├── 3D Volume DVR: Screen-aligned raymarching with trilinear filtering & jitter
│   └── 3D Mesh / Point Cloud with OIT:
│       ├── Pass 1 (fs_opaque): Writes depth for opaque texels (alpha >= 0.995)
│       ├── Pass 2 (fs_oit): Accumulates weighted color (Rgba16Float) + revealage (R16Float)
│       └── Pass 3: Fullscreen composite blit into egui canvas rect
└── 4. Coastline geographic vector overlay projected into dataset domain

Phase 5: Canvas Compositor, Inspection & Export
│
├── 1. Dynamic plot axes & ticks rendered with coordinate-aware formatting
├── 2. Floating colorbar panels rendered per active layer
├── 3. Hover sampling & reticle crosshair (cell extraction or 3D volume raycast)
├── 4. Interactive ROI crop overlay and camera capture flash
└── 5. Export engine: figures encoded to PNG, WebP (Display P3), SVG, PDF, or clipboard
```

---

## 3. Directory Structure & Key Subsystems

```
src/
├── app/                  # Application state, orchestration, and paint dispatch
│   ├── block_loading/    # Hyperslab requests, async coordination, projection
│   ├── canvas/           # Decoupled canvas engine: viewport, interactions, axes, overlays
│   ├── controllers/      # Per-plot-type controllers (Line, Volume, Heatmap, Mesh, PointCloud)
│   ├── layers/           # LayerStack, Layer, alignment, styles, sources
│   ├── pipeline/         # GPU pipeline rebuilds, aspect ratio math, uniform assembly
│   ├── state/            # Decomposed states: NavigationState, PlaybackState, PlotConfigs, UiLayoutState
│   ├── actions.rs        # AppAction event-driven state mutation
│   ├── data_loading.rs   # Store inspection delegation
│   ├── export_lifecycle.rs # In-flight figure export and screenshot processing
│   ├── floating.rs       # Floating overlay panels and colorbars
│   ├── overlays.rs       # Multi-layer overlay alignment and lifecycle
│   └── ui.rs             # eframe::App immediate-mode main loop (< 190 lines)
├── data/                 # Format-agnostic N-D data system & storage engine
│   ├── backends/         # Storage backends: geotiff, zarr, icechunk, netcdf, procedural
│   ├── blocks/           # LRU BlockCache, BlockPrefetcher, CoordinateLoader, BlockStore trait
│   ├── calibration/      # NetCDF scale_factor, add_offset calibration
│   ├── codecs/           # Pure-Rust Blosc and Zstandard codec plugins for zarrs
│   ├── coordinates/      # Topology, coordinate ordering, DGGS, HEALPix, regular axes
│   ├── dataset/          # DatasetManager, StoreHandle, DataSource factory
│   ├── metadata/         # DatasetMetadata, VariableInfo, CoordValues (Regular/Values/Labels)
│   ├── procedural/       # Synthetic HEALPix spherical and 3D datasets
│   ├── render/           # MatrixData, VolumeData, downsampled MatrixPyramid
│   ├── slicing/          # 1D/2D/3D hyperslab extraction, RGB/CMYK composites
│   └── octant_block.rs   # In-memory resident N-D hyperslab representation
├── plots/                # WGPU renderers, WGSL shaders, and graphics pipelines
│   ├── coastline/        # Coastline 2D & 3D vector boundary overlays
│   ├── line/             # 1D line payload generation and multi-series curves
│   ├── oit/              # Weighted-blended order-independent transparency passes
│   ├── volume/           # 3D DVR raymarching, 8-tap filter, jitter, lighting, textures
│   ├── colormap_atlas.rs # Shared 256×K RGBA8 texture atlas bound at @group(1)
│   ├── heatmap.rs        # 2D heatmap matrix renderer
│   ├── mesh.rs           # 3D displaced mesh renderer (Surface & Sphere)
│   ├── point_cloud.rs    # 3D billboarded point cloud renderer
│   └── traits.rs         # PlotRenderer polymorphic trait and HoverSample
├── ui/                   # egui GUI panels, widgets, and canvas overlays
│   ├── about/            # About modal and virtualized license viewer
│   ├── bottom_bar/       # Animation playback timeline, step sliders, status badges
│   ├── catalog/          # Dataset catalog modal with category filters
│   ├── colorbar/         # Multi-layer colorbar panels, tick formatting, series bar
│   ├── colormap/         # Colormap picker, custom colorgrad editor, swatch atlas
│   ├── hover/            # Canvas crosshairs, value inspection, multi-layer rows
│   ├── settings/         # Rendering controls, clipping, layer stack manager
│   ├── store/            # Left panel for store selection, URI inputs, cache stats
│   ├── top_bar/          # Navigation header with responsive priority collapse
│   ├── variables_panel/  # Dimension role selectors (X, Y, Z, Anim), range sliders
│   └── axes.rs           # Dynamic plot axes, grid lines, and tick generators
├── utils/                # Colormaps, coordinate discovery, math, and diagnostics
│   ├── colormap/         # LUT evaluation, colormap catalog, opacity alpha curves
│   ├── grid.rs           # Axis flipping and orientation logic
│   └── grid_flips.rs     # Spatial dimension reorientation hints
├── export/               # Figure export engine: PNG, WebP (Display P3), SVG, PDF, clipboard
└── catalog/              # Static built-in dataset definitions (GeoTIFF, Zarr, Icechunk)
```

---

## 4. Modular Design Principles & Human Maintainability

Octant's architecture follows strict separation of concerns, high cohesion, and the Open-Closed Principle (OCP). Each file is constrained to `< 250 lines` and each function to `< 50 lines`.

### The 4 Modular Pillars

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                           Architectural Pillars in Octant                        │
└──────────────────────────────────────────────────────────────────────────────────┘

  1. State Decomposition (src/app/state/)
     ├── NavigationState: Camera orbit, 2D pan/zoom, resets, interaction flags
     ├── PlaybackState: Timestep navigation, FPS, playback loop, timers
     ├── PlotConfigs: Typed configs (Line, Volume, Mesh, PointCloud)
     └── UiLayoutState: Panel visibility toggles, search, layout widths

  2. Decoupled Canvas Engine (src/app/canvas/)
     ├── viewport.rs: Aspect ratio scaling, screen-space rect calculation
     ├── interactions.rs: Gesture dispatching delegating to PlotController
     ├── axes.rs: Coordinate tick generation, domain resolution, title placement
     ├── overlays.rs: Capture flash, interactive ROI crop tool, drag cue
     └── mod.rs: Single-point canvas coordinator (< 90 lines)

  3. Polymorphic Plot Controllers (src/app/controllers/)
     ├── PlotController trait: View reset, navigation math, capabilities
     ├── Singletons: HeatmapController, LineController, VolumeController, etc.
     └── controller_for(): Zero-allocation O(1) controller lookup

  4. Isolated Frame Lifecycle (src/app/export_lifecycle.rs & actions.rs)
     ├── export_lifecycle.rs: In-flight figure encoding and screenshot captures
     └── actions.rs: Event-driven AppAction dispatching
```

### Why Feature Additions Touch Only 1–3 Files

By decoupling presentation, navigation, and configuration from the top-level application struct, changes are strictly localized:
- **Adding a new plot type**: Implement `PlotRenderer` in `src/plots/`, implement `PlotController` in `src/app/controllers/`, and add its typed config to `PlotConfigs`.
- **Modifying camera navigation or gestures**: Update the specific controller in `src/app/controllers/` without touching `ui.rs`, rendering pipelines, or other plot types.
- **Adjusting viewport scaling or axes**: Handled entirely inside `src/app/canvas/viewport.rs` or `src/app/canvas/axes.rs`.

---

## 5. Core Architectural Subsystems

```text
                    ┌────────────────────────────────────────────────────────┐
                    │                   Subsystem Architecture               │
                    └────────────────────────────────────────────────────────┘

     ┌───────────────────────────────────────────────────────────────────────────────┐
     │                       App State Subsystems (src/app/state/)                   │
     │  - NavigationState (camera orbit, pan, zoom, gestures)                        │
     │  - PlaybackState (animation step, fps, loop, timer)                           │
     │  - UiLayoutState (overlay toggles, panel positions, search, theme)            │
     │  - PlotConfigs (typed LinePlotConfig, VolumePlotConfig, MeshPlotConfig, etc.) │
     │  - LayerStack (Base + Overlays)                                               │
     └──────────────────────────────────────┬────────────────────────────────────────┘
                                            │ Dispatches to active
                                            v
     ┌───────────────────────────────────────────────────────────────────────────────┐
     │            Polymorphic Plot Controllers (src/app/controllers/)                │
     │                                                                               │
     │  pub trait PlotController: Send + Sync {                                      │
     │      fn plot_type(&self) -> PlotType;                                         │
     │      fn is_3d(&self) -> bool;                                                 │
     │      fn reset_view(&self, nav: &mut NavigationState);                         │
     │      fn handle_drag(&self, nav: &mut NavigationState, delta: Vec2);           │
     │      fn handle_scroll(&self, nav: &mut NavigationState, scroll: f32, ...);    │
     │      fn draws_coastlines(&self) -> bool;                                      │
     │      fn draws_categories(&self) -> bool;                                      │
     │  }                                                                            │
     └──────────────────────────────────────┬────────────────────────────────────────┘
                                            │
               ┌────────────────────────────┼───────────────────────────┐
               v                            v                           v
     ┌───────────────────┐        ┌───────────────────┐       ┌───────────────────┐
     │ HeatmapController │        │  LineController   │       │ VolumeController  │
     │ (src/app/         │        │ (src/app/         │       │ (src/app/         │
     │  controllers/     │        │  controllers/     │       │  controllers/     │
     │  heatmap.rs)      │        │  line.rs)         │       │  volume.rs)       │
     └───────────────────┘        └───────────────────┘       └───────────────────┘
               │                            │                           │
               └────────────────────────────┼───────────────────────────┘
                                            v
     ┌───────────────────────────────────────────────────────────────────────────────┐
     │                 Decoupled Canvas Subsystem (src/app/canvas/)                  │
     │  - viewport.rs: compute_viewport_uniforms (aspect scaling, rect transforms)   │
     │  - interactions.rs: handle_canvas_interactions (delegates to PlotController)  │
     │  - axes.rs: draw_canvas_axes (2D / 1D axis labels and ticks)                  │
     │  - overlays.rs: draw_canvas_overlays (capture flash, crop frame, drag cue)    │
     │  - mod.rs: render_canvas entrypoint (< 90 lines)                              │
     └───────────────────────────────────────────────────────────────────────────────┘
```

### Detailed Subsystem Responsibilities

#### 1. Componentized App States (`src/app/state/`)
The fields of `OctantApp` are decomposed into cohesive sub-states:
- `NavigationState`: 2D `heatmap_pan`, `heatmap_zoom`, `line_pan`, `line_zoom`, 3D `sphere_rotation_x/y`, `sphere_zoom`.
- `PlaybackState`: `is_playing`, `playback_fps`, `loop_playback`, `last_step_time`, `current_timestep`.
- `UiLayoutState`: `show_left_panel`, `show_variables_overlay`, `show_settings_panel`, `panel_positions`, search query.
- `PlotConfigs`: Holds typed configuration structs (`LinePlotConfig`, `VolumePlotConfig`, `MeshPlotConfig`, `PointCloudPlotConfig`) instead of loose primitive fields.

#### 2. Plot Controllers (`src/app/controllers/`)
Pure GPU graphics pipelines live in `src/plots/`, while interaction handling, navigation math, and plot capabilities live in `src/app/controllers/`:
- `src/app/controllers/traits.rs`: Defines the `PlotController` trait.
- `src/app/controllers/heatmap.rs`: Encapsulates 2D flatmap planar drag and zoom.
- `src/app/controllers/line.rs`: Encapsulates 1D profile drag and zoom.
- `src/app/controllers/volume.rs`: Encapsulates 3D raymarching orbit and zoom.
- `src/app/controllers/mesh.rs`: Encapsulates Surface and Sphere 3D mesh orbit and zoom.
- `src/app/controllers/point_cloud.rs`: Encapsulates 3D point cloud orbit and zoom.
- `src/app/controllers/mod.rs`: Singleton dispatcher `controller_for(plot_type) -> &'static dyn PlotController`.

#### 3. Decoupled Canvas Engine (`src/app/canvas/`)
`src/app/ui.rs` is reduced from 660+ lines to under 190 lines, with canvas orchestration separated into focused submodules (< 120 lines each):
- `src/app/canvas/viewport.rs`: Viewport dimensions, canvas rect computation, GPU aspect ratio scaling.
- `src/app/canvas/interactions.rs`: Mouse drag, wheel zoom, double-click reset, delegating to `PlotController`.
- `src/app/canvas/axes.rs`: Computes plot rect scaling, axis domains, and renders ticks/titles.
- `src/app/canvas/overlays.rs`: Capture flash, interactive ROI crop overlay, drag-and-drop cue.
- `src/app/canvas/mod.rs`: `render_canvas` entrypoint.

---

## 6. Guidelines for Extending Octant

### 6.1 Adding a New Storage Backend (e.g. HDF5, GeoParquet, Cloud Stores)
1. Create `src/data/backends/your_backend/`.
2. Implement the [`BlockStore`](https://github.com/lazarusA/octant/blob/main/src/data/blocks/store.rs) trait:
   - `backend_name(&self) -> &str`
   - `inspect(&self) -> Result<DatasetMetadata, BlockStoreError>` (Metadata-only inspection)
   - `variable_coordinates(&self, variable: &VariableInfo) -> Result<HashMap<String, CoordValues>, BlockStoreError>` (Lazy dimension coordinate retrieval)
   - `fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError>`
   - `fetch_block_with_progress(&self, request: &SliceRequest, on_progress: ProgressCallback) -> Result<OctantBlock, BlockStoreError>`
3. Wire the source detection in `src/utils/store.rs` and `src/app/data_loading.rs`.

### 6.2 4-Step Plug-in Pattern for Adding Any New Plot Type
Octant's decoupled architecture ensures that adding a new visualization type never requires modifying the central immediate-mode UI loop (`src/app/ui.rs`), the canvas engine (`src/app/canvas/`), or the export lifecycle (`src/app/export_lifecycle.rs`). Every plot type follows a strict 4-step plug-in contract:

```text
Step 1: Configuration & Variant
  ├── Add typed configuration struct to src/app/state/plot_configs.rs (e.g., TrajectoryConfig)
  └── Register variant in PlotType enum (src/plots/mod.rs)

Step 2: GPU Pipeline & Renderer (src/plots/)
  ├── Implement PlotRenderer trait in src/plots/your_plot.rs:
  │   ├── update_data(&self, queue, data: &RenderData) -> GPU buffer allocations
  │   ├── paint(&self, ui, rect, params: &PlotRenderParams) -> Submit egui-wgpu callback
  │   └── inspect_hover(&self, pointer_pos, rect, data, params) -> Option<HoverSample>
  └── Write WGSL shader in src/plots/shaders/your_plot.wgsl

Step 3: Plot Controller & Event Management (src/app/controllers/)
  ├── Implement PlotController trait in src/app/controllers/your_plot.rs:
  │   ├── handle_input(&self, app, response) -> Custom gesture, drag, or hover picking
  │   ├── on_staged_selection_change(&self, app) -> Dimension validation
  │   ├── on_data_loaded(&self, app, layer_id) -> Translation into GPU buffers
  │   └── update_uniforms(&self, app, layer_id) -> Prep PlotRenderParams from PlotConfigs
  └── Register in controller_for(plot_type) singleton dispatcher (src/app/controllers/mod.rs)

Step 4: Pipeline Paint Callback Wiring
  └── Add match arm in src/app/pipeline/paint_layer.rs and LayerRenderers (src/app/layers/renderers.rs)
```

### 6.3 Supporting Complex & Non-Grid Workflows (Lagrangian Trajectories, Streamlines & Particle Tracks)
Non-Eulerian data (such as ocean drifter trajectories, atmospheric weather balloons, aircraft flight paths, and Lagrangian particle advection) differs fundamentally from regular hyperslab tensors. Octant accommodates these workflows seamlessly through its decoupled state and controller model:

1. **Non-Grid Coordinate Trajectory Ingestion**:
   - Trajectory records represent sequences of 4D/5D tuples $(x_t, y_t, z_t, v_t)$ evolving over time.
   - In `PlotController::on_data_loaded`, arriving block records or sparse tables are decoded directly into path vertices or GPU instance buffers.
2. **GPU Vertex Pulling & Analytical Ribbons**:
   - In accordance with Octant's zero-allocation GPU rendering guidelines, do not generate polygonal tube meshes on the CPU.
   - Pass trajectory nodes as a GPU storage buffer (`@group(0) @binding(1) var<storage, read> path_nodes: array<PathPoint>;`).
   - Synthesize camera-facing 3D ribbon strips or billboard arrows directly inside the WGSL vertex shader (`vs_main`).
3. **Temporal Scrubbing & Track History**:
   - Trajectory paths synchronize with global animation using [`PlaybackState`](https://github.com/lazarusA/octant/blob/main/src/app/state/playback.rs) (`app.playback.current_timestep` and `app.playback.last_step_time`).
   - The trajectory controller can render the entire track history with a leading marker or filter path segments up to the current timestamp.
4. **Interactive Picking & Path Inspection**:
   - Custom pointer selection, waypoint inspection, or nearest-point Euclidean distance tests live inside `PlotController::handle_input` and `PlotRenderer::inspect_hover`.
   - Camera auto-tracking (centering the viewport on an active particle) manipulates [`NavigationState`](https://github.com/lazarusA/octant/blob/main/src/app/state/navigation.rs) (`app.nav`) cleanly without borrow conflicts.

---

## 7. Layer & Multi-Scale Roadmap Status

### Current Status
- **Done: Plotted State in `LayerStack`**: Staged UI selection lives in `selected: VariableSelection`; Plot copies it to the base layer.
- **Done: Same-Grid Overlays**: `alignment.rs` classifies overlays against the base layer (`SameGrid`, `Geo`, `IndexOnly`, `Incompatible`). Overlays render via `LayerStack` with individual opacity, colormaps, and colorbars.
- **Done: 1D Line Profiles & Multi-Series**: Multi-series line plots (`All Lines Series`) with custom series colorbars and strided hover sampling.
- **Done: Shared Colormap Atlas**: 256×K RGBA8 texture atlas shared across CPU evaluation and all GPU pipelines at `@group(1)`.
- **Done: Order-Independent Transparency (OIT)**: Weighted blended transparency for overlapping meshes and translucent point clouds.
- **Done: Modular State & Canvas Engine**: Decomposed app states (`NavigationState`, `PlaybackState`, `PlotConfigs`, `UiLayoutState`), decoupled canvas engine (`src/app/canvas/`), and polymorphic plot controllers (`src/app/controllers/`).

### Next Roadmap Objectives
1. **Multi-Plot Overlays**:
   - Extend overlays beyond heatmaps to 3D volumes (multi-channel texture raymarching) and surfaces.
2. **Cross-Dataset Geographic Overlays**:
   - Render `Alignment::Geo` overlays with normalized longitude conventions and bounding box placement.
3. **Multi-Scale Viewport Level-of-Detail**:
   - Progressive multiscale loading for large-scale OME-NGFF and GeoTIFF overviews.
4. **Derived Sources & Operations (`Source::Derived`)**:
   - CPU/GPU algebraic expressions and vector field synthesis across variables (e.g. wind speed magnitude $\sqrt{u^2 + v^2}$ or vorticity $\nabla \times \vec{v}$).
5. **Lagrangian Trajectory & Vector Streamline Subsystem**:
   - Generalize `Source` in `src/app/layers/source.rs` from `Source::Variable(VariableSelection)` to `Source::Trajectory(TrajectorySelection)`.
   - Implement `TrajectoryRenderer` and `TrajectoryController` with GPU vertex pulling for high-performance particle paths.
