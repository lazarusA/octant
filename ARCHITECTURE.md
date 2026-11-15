# Octant architecture

Octant is a viewer for N-dimensional datasets (Zarr, Icechunk, OME-Zarr, GeoTIFF, NetCDF, HDF5,
MATLAB v7.3, GRIB2), local or in cloud object storage. It is written in Rust (2024 edition) with
[`egui`](https://github.com/emilk/egui) for the interface and [`wgpu`](https://wgpu.rs/) for
rendering, and runs as a desktop app and in the browser (WebAssembly).

This file is the map. Each subsystem's types, files and rules are described in its skill reference
under [`.agents/skills/`](.agents/skills/), and the coding rules in [AGENTS.md](AGENTS.md).

## Two rules behind the design

- **Read only what is on screen.** A plotted layer asks for one window; the reader fetches only the
  chunks, shard pieces, TIFF tiles or GRIB messages that window touches, from the pyramid level that
  fits the canvas.
- **Draw on the GPU.** Values are uploaded once per change; shaders apply colormaps, lighting,
  channel blending and transparency.

## Data flow

```text
 Inputs: panels · canvas gestures · keyboard · typed or spoken commands · share links
   │  AppAction (app/actions): the UI, voice and links call the same app methods
   ▼
 Staged selection ──── Plot ────▶ LayerStack: base + up to 4 overlays
 (app/state)                      (app/layers)
                                    │  one window per layer, playback steps ahead
                                    │  (app/block_loading)
                                    ▼
 BlockCache + prefetcher (data/blocks) ── miss ──▶ fetch runtime ──▶ BlockStore ──▶ source
   ▲                                               desktop: Tokio     data/backends   local, S3, GCS,
   │                                               browser: workers                   Azure, HTTP
   └────────────── decoded OctantBlock (values in Arc<[f32]>) ◀─────────────────────────┘
                                    │
                                    ▼
 Arrival: orient (block_orientation), calibrate, slice, composite ──▶ LayerData
 (app/block_loading, data/slicing)                                    MatrixData · VolumeData · lines
                                    │
                                    ▼
 Pipeline (app/pipeline): rebuild what changed: uniforms, GPU channel bake, thinned mesh grid
                                    │
                                    ▼
 Renderers (plots/): heatmap · line · surface · sphere · volume · point cloud, plus labels,
 coastlines, colormap atlas, order-independent transparency, frame cache
                                    │
                                    ▼
 Canvas (app/canvas, ui/): camera, axes, colorbars, hover readout, slicer gizmo ──▶ export
   │
   └─ view at rest ──▶ multiscale (data/multiscale, app/detail): a finer tile over the view
```

Metadata arrives first (`BlockStore::inspect`) and fills the panels; coordinates load in the
background (`data/blocks/coord_loader.rs`), and code falls back to indices until they arrive.
Browsing or staging another variable never changes the plot: only Plot copies the staged selection
into the `LayerStack`, and only plotted layers request data.

## Subsystems

| Area | Where | What | Reference |
|---|---|---|---|
| Sources and readers | `src/data/dataset/`, `src/data/backends/` | `DataSourceKind`, `SourceFactory`, one `BlockStore` per format; NetCDF, HDF5, MATLAB and GRIB share the `NcReader` seam (`backends/netcdf/reader.rs`) | [data.md](.agents/skills/octant-data-engine/data.md) |
| Blocks and caching | `src/data/blocks/`, `src/data/worker/` | `BlockCache` (LRU), prefetcher, coordinate loader, memory budget; browser data workers and their job protocol | [data.md](.agents/skills/octant-data-engine/data.md) |
| Metadata and coordinates | `src/data/metadata/`, `src/data/coordinates/`, `src/data/crs/`, `src/data/calibration/` | variables and dimensions, grids and topologies (DGGS, HEALPix), CRS discovery, scale/offset and fill values | [data.md](.agents/skills/octant-data-engine/data.md) |
| OME-Zarr and pyramids | `src/data/ome/`, `src/data/multiscale/`, `src/app/detail/` | OME images, labels, plates; OME, GeoZarr and GeoTIFF level stacks; detail tiles | [data.md](.agents/skills/octant-data-engine/data.md) |
| Slicing | `src/data/slicing/`, `src/data/block_orientation.rs` | 1D/2D/3D extraction, RGB, CMYK and multichannel composites, stored-order indexing | [data.md](.agents/skills/octant-data-engine/data.md) |
| Catalog | `src/catalog/` | built-in public datasets | [catalog.md](.agents/skills/octant-data-engine/catalog.md) |
| App state and actions | `src/app/state/`, `src/app/actions/`, `src/app/share/` | staged selection, navigation, playback, plot configs, slicer; `AppAction` dispatch; share links | [app.md](.agents/skills/octant-ui-egui/app.md) |
| Layers and loading | `src/app/layers/`, `src/app/block_loading/` | `LayerStack`, alignment, styles; requests, arrival, projection, playback buffer, volume ring | [app.md](.agents/skills/octant-ui-egui/app.md) |
| Canvas and controllers | `src/app/canvas/`, `src/app/controllers/`, `src/app/pipeline/` | viewport, gestures, `PlotController` per plot type, paint dispatch | [app.md](.agents/skills/octant-ui-egui/app.md) |
| Renderers | `src/plots/` | one renderer per plot type, WGSL shaders, camera, channel bake, OIT, frame cache, buffer limits | [plots.md](.agents/skills/octant-rendering-wgpu/plots.md) |
| Colormaps | `src/utils/colormap/` | catalog, registry, shared GPU atlas, opacity curves | [colormaps.md](.agents/skills/octant-rendering-wgpu/colormaps.md) |
| Export | `src/export/` | PNG, JPEG, WebP, SVG, PDF, clipboard, browser download | [export.md](.agents/skills/octant-rendering-wgpu/export.md) |
| Interface | `src/ui/` | panels, toolbars, hover, colorbars, keyboard, toasts, icons, hero | [ui.md](.agents/skills/octant-ui-egui/ui.md) |
| Languages | `src/i18n/` | 15 languages, templates, bundled CJK font subsets | [i18n.md](.agents/skills/octant-ui-egui/i18n.md) |
| Voice | `src/voice/` | capture, Whisper speech to text (desktop and web worker), feeding `AppAction` | [app.md](.agents/skills/octant-ui-egui/app.md) |
| Build and release | `Cargo.toml`, `.github/` | release profile, wasm size budgets, installers, licenses | [build.md](.agents/skills/rust-workflows/build.md) |

## Desktop and browser

| Stage | Desktop | Browser |
|---|---|---|
| Fetch and decode | shared Tokio runtime (`utils/executor.rs`) and prefetch threads | data workers run the whole loader (`data/worker/`); the page gets decoded blocks |
| NetCDF and HDF5 | netCDF-C for local files, `hdf5-pure` for remote NetCDF-4 | `hdf5-pure` in the worker |
| GRIB2 | pure Rust reader, JPEG2000 fields included | same reader, JPEG2000 fields off |
| Local files | paths on disk | picked or dropped files and folders, served as `local://` (`backends/local/`) |
| GPU | Vulkan, Metal or Direct3D 12 | WebGPU, with a WebGL2 fallback |

## Adding a format

1. Write the reader in `src/data/backends/<format>/` and implement `BlockStore`
   ([store.rs](src/data/blocks/store.rs)): `backend_name`, `variables`, `inspect`,
   `variable_coordinates` (when coordinates load after the dataset opens) and `fetch_block`. A
   file format with NetCDF-like variables can instead implement `NcReader` and reuse the NetCDF
   backend, as GRIB does.
2. Store values in `Arc<[f32]>` and index blocks in stored order through
   `src/data/block_orientation.rs`.
3. Add one `DataSourceKind` variant ([source.rs](src/data/dataset/source.rs)), no catch-all, and
   match it in `data/dataset/factory.rs`, `app/state/store_kind.rs`, `ui/dataset/manager.rs`
   (`source_kind_icon`), `utils/path.rs` and `data/backends/web.rs` (`WebFormat::of`).
4. Check it in the browser: the store must build for `wasm32` and open inside a data worker.

## Adding a plot type

1. Add a `PlotType` variant ([plots/mod.rs](src/plots/mod.rs)) and its `PlotKey` name in every
   language file.
2. Write the renderer in `src/plots/` and its shader in `src/plots/shaders/`. Read the camera from
   `plots/camera.rs`; uniform structs are `#[repr(C)]` `Pod` and get a check in
   `uniform_layout_tests.rs`.
3. Implement `PlotController` ([traits.rs](src/app/controllers/traits.rs)), or reuse the planar or
   orbit controller, and add its arm to `controller_for` (`app/controllers/mod.rs`).
4. Give it a slot in `LayerRenderers` (`app/layers/renderers.rs`), a paint arm in
   `app/pipeline/paint_layer.rs` and a data projection in `app/block_loading/projection.rs`.
5. Follow the compiler: `PlotType` matches are exhaustive, so the hover, settings, plot type
   picker, slicer, share links and actions show what is left.

## Not built yet

Ideas the structure leaves room for, with no code yet: map projections and reprojection,
curvilinear grids drawn at their true position (`CoordinateGrid::Curvilinear2D` exists but nothing
builds it), out-of-core bricked volumes, derived variables (such as wind speed from `u` and `v`),
and particle trajectories drawn by GPU vertex pulling.
