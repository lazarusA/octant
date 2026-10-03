# Octant Agent Rules

When developing and reviewing code in this repository:

1. **Rust Quality & Checks**:
   - Adhere to idiomatic Rust (Rust 2024 edition).
   - Enforce borrow-over-clone (`&[T]` over `&Vec<T>`, `&str` over `&String`).
   - Forbid `unwrap()` in production code (use `?`, `let Some(...) = ... else`, or `f32::total_cmp`).
   - Use poison-resilient lock handling (`if let Ok(guard) = ...` or `.unwrap_or_else(|p| p.into_inner())`).
   - Always use checked arithmetic when computing multi-dimensional tensor shape volumes (`shape.iter().try_fold(...)`).
   - Run `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`.
   - For C-library/FFI test suites (e.g. NetCDF), serialize file creation with static mutex test locks to prevent concurrent non-reentrant IO collisions.

2. **Modular Architecture & Subsystem Layout**:
   - Follow the Open-Closed Principle (OCP): decompose monolithic modules into single-purpose submodules (< 250 lines per file, < 50 lines per function).
   - **Catalog Subsystem (`src/catalog/`)**:
     - `entries.rs`: Built-in static catalog tables (GeoTIFF, Zarr, Icechunk, Procedural).
     - `types.rs`: `CatalogEntry`, `CatalogCategoryFilter`, `CatalogProvider` trait.
     - `tests.rs`: Category filtering, projection capability, and JSON serialization tests.
     - `mod.rs`: Re-exports and `get_catalog_entries`.
   - **Data Engine Subsystems (`src/data/`)**:
     - **Blocks Engine (`src/data/blocks/`)**: `cache.rs` (LRU memory cache), `key.rs` (`BlockKey`), `loader.rs` (async background workers), `prefetch.rs` (`BlockPrefetcher`), `request.rs` (`SliceRequest`), `store.rs` (`BlockStore` trait), `summary.rs` (`BlockSummary`), `tests.rs`, `mod.rs`.
     - **Backends (`src/data/backends/`)**:
       - `geotiff/`: `reader.rs` (async COG header discovery & pooled range reader), `slice.rs`, `tests.rs`, `mod.rs`.
       - `http/`: `fetch.rs`, `mod.rs` (Browser `window.fetch` and Desktop connection-pooled `get_http_client` range requests).
       - `coord_bounds/`: `cache.rs`, `candidates.rs`, `discover.rs`, `extract.rs`, `tests.rs`, `mod.rs` (Unified coordinate boundary resolution & global cache).
       - `zarr/`: `block.rs`, `generic.rs`, `slice.rs`, `storage.rs`, `store.rs`, `zstd_shim.rs`, `wasm/` (`inspect.rs`, `loader.rs`, `preload.rs`, `store.rs`, `mod.rs`), `mod.rs`.
       - `icechunk/`: `native.rs`, `wasm/` (`discovery.rs`, `header.rs`, `inspect.rs`, `loader.rs`, `preload.rs`, `store.rs`, `tests.rs`, `mod.rs`), `mod.rs`.
       - `netcdf/`: `attrs.rs`, `coords.rs`, `desktop.rs`, `inspect.rs`, `slice.rs`, `wasm.rs`, `tests.rs`, `mod.rs`.
       - `procedural/`: `healpix.rs`, `healpix_meta.rs`, `inspect.rs`, `slice_2d.rs`, `slice_3d.rs`, `store.rs`, `tests.rs`, `mod.rs`.
     - **Codecs (`src/data/codecs/`)**: `blusc_plugin.rs` (Pure Rust Blosc `zarrs` codec plugin), `zstd_plugin.rs` (Pure Rust Zstandard `zarrs` codec plugin), `normalize.rs` (v3 metadata ordering & pipeline normalization), `tests.rs`, `mod.rs`.
     - **Coordinates (`src/data/coordinates/`)**: `naming.rs` (dimension role inference & ASCII case-insensitive searches), `detection.rs`, `dggs.rs`, `lut.rs`, `ordering.rs`, `regular.rs`, `same_geometry.rs`, `search.rs`, `topology.rs`, `types.rs`, `healpix/`, `topologies/`, `impl_topology/`.
     - **Data Slicing (`src/data/slicing/`)**: `common.rs` (math & range clamping), `coords.rs` (sliced coordinate extraction), `copy.rs` (strided & contiguous copy routines), `slice_1d.rs` (1D/0D slabs), `slice_2d.rs` (2D hyperslabs), `slice_3d.rs` (3D volumetric slabs).
     - Store tensor values in `Arc<[f32]>` for $O(1)$ zero-copy sharing between cache and render pipelines.
   - **WGPU Renderers (`src/plots/`)**:
     - All concrete plot renderers (`HeatmapRenderer`, `LineRenderer`, `Mesh3DRenderer`, `VolumeRenderer`, `PointCloudRenderer`) must implement the polymorphic `crate::plots::traits::PlotRenderer` trait.
     - Uniform parameter structs (such as `Mesh3DUniformParams`) must be zero-allocation `Copy` structs; never clone large coordinate vectors or data structures inside paint loops.
     - For all structured, discrete global (HEALPix, ICON, Cubed-Sphere), and unstructured grid formats (UGRID, MPAS), use zero-allocation GPU instancing or GPU vertex pulling. Reserve CPU mesh generation strictly for non-grid geometry (GIS vector polygons, streamlines, Marching Cubes CAD export, UI labels and annotations).
   - **Export Engine (`src/export/`)**:
     - Submodules: `raster.rs` (PNG, JPEG, WebP, Display P3 chunk injection), `vector.rs` (SVG, PDF), `clipboard.rs` (native file manager reveal & clipboard).
   - **UI Subsystems (`src/ui/`)**:
     - `src/ui/about/`: Modal window (`types.rs`, `overview.rs`, `icons.rs`, `mod.rs`).
     - `src/ui/toolbar/`: Shared width-driven collapse for both bars (`ItemWidths`, `BarItem`, `CompactFlags`, `compute_compact`, `SEPARATOR_WIDTH`; `tests.rs`, `mod.rs`). Bars collapse items to icon-only (data-only items hide) in a fixed priority order instead of using overflow menus.
     - `src/ui/top_bar/`: Top navigation bar (`layout.rs` items & collapse order, `items.rs` drawing & widths, `tests.rs`, `mod.rs`).
     - `src/ui/bottom_bar/`: Playback bar (`layout.rs` items & collapse order, `items.rs` button items, `labels.rs` date & badge items, `timeline.rs` per-frame axis labels, `tests.rs`, `mod.rs`).
     - `src/ui/catalog/`: Dataset preset modal (`header.rs`, `filters.rs`, `card.rs`, `list.rs`, `mod.rs`).
     - `src/ui/color_picker/`: Color swatch and popup selector (`popup.rs`, `shape.rs`, `widget.rs`, `tests.rs`, `mod.rs`).
     - `src/ui/colorbar/`: `ticks.rs` (scientific tick generation), `handles.rs` (input boxes and clip triangles), `mod.rs` (overlay coordinator).
     - `src/ui/crop_overlay/`: Canvas crop bounding box (`handles.rs`, `toolbar.rs`, `tests.rs`, `mod.rs`).
     - `src/ui/brand/`: Brand marks (`mod.rs` shared cube `FACE_SHADES`, `darken`, theme-aware `face_colors`; `wordmark.rs` block `Wordmark` widget: "OCTANT" as 5x5 cube cells with whole-pixel snapping, and flat drop shadows below 7 physical px per cell; `tests.rs`). The hero cube and the wordmark must share these shades.
     - `src/ui/hero/`: Landing screen and source intake (`chips.rs`, `feedback.rs`, `intake.rs`, `landing.rs`, `state.rs`, `style.rs` gutter/width/font metrics, `widget.rs`, `mod.rs`).
     - `src/ui/hover/`: `callout.rs` (reticle & elbow leader arm anchored to the placed card edge), `camera.rs`, `enrich.rs`, `entries.rs`, `entries_1d.rs`, `entries_2d.rs`, `entries_3d.rs`, `field.rs` (`HoverField` label/value rows), `format.rs`, `hit.rs`, `raycast_sphere.rs`, `raycast_surface.rs`, `raycast_volume.rs`, `sample_1d.rs`, `sample_2d.rs`, `mod.rs`; `card/` hover card (`model.rs` `HoverCard`/`HoverValue` with stack-buffer value formatting, `layout.rs` typography & exact measurement, `flow.rs` coordinate `label value` pairs in wrapping rows split by hairline dividers (card shrinks to its widest row), `place.rs` in-canvas diagonal `Placement`/`Side`, `paint.rs`, `sheet.rs` `cargo test --lib hover_card_contact_sheet -- --ignored` review sheet, `tests.rs`, `mod.rs` `show_card`). Measure the card before placing it, and paint the leader before the card on the same tooltip layer so the card covers the leader's end. The title is the variable's `long_name` when available, else its name; units use the value font size in a muted color.
     - `src/ui/icons/`: Procedural vector icons (`canvas/` shared `IconCanvas` drawing surface: `mod.rs` grid mapping, pixel snapping & whole-pixel strokes, `draw.rs` primitives, `tokens.rs` `key` keylines/`Weight`/`Shade`, `geom.rs` convexity & dedup; `nav.rs`, `playback.rs`, `plots.rs`, `palette.rs` heatmap & colormap brush, `files.rs` dataset & folders, `status.rs` padlocks/bolt/hourglass/scissors, `marks.rs` check/cross/warning/info/bullet/chevron, `store.rs` drawing routines; `paint.rs` dispatch; `meta.rs` names & categories; `style.rs` `IconSize`/`IconTone` (`IconTone::rgb` for multi-color icons)/stroke weights; `ext.rs` `UiIconExt`; `button.rs` `ToolbarButton`; `sheet.rs` icon contact sheet (`cargo test --lib contact_sheet -- --ignored` writes `target/icon_sheets/*.png`, including `wordmark.png`, via the test-only CPU renderer `src/ui/test_render.rs`); `tests.rs`, `mod.rs`). Every icon draws through `IconCanvas` (`fn draw_x(c: &IconCanvas)`): use its `key` keylines, `Weight`/`Shade` tokens and primitives instead of raw `Painter` calls or ad-hoc widths and alphas, fill only convex polygons (split concave shapes), draw each edge once, and drop secondary detail when `c.compact()` (below 16 px).
     - `src/ui/settings/`: Dedicated settings submodules (`clipping.rs`, `coastline.rs`, `export.rs`, `plot_2d.rs`, `plot_3d.rs`, `plot_options.rs`, `resampling.rs`, `mod.rs`).
     - `src/ui/variables_overlay/`: Floating variable search modal (`item.rs`, `tree.rs`, `mod.rs`; the search row is `UiIconExt::search_field`).
     - `src/ui/variables_panel/`: Docked sidebar inspector (`info.rs`, `mod.rs`) and dimension slider controls (`dimension_slider/`: `defaults.rs`, `double_slider.rs`, `metrics.rs`, `roles.rs`, `slice_req.rs`, `slider_row.rs`, `mod.rs`).
     - `src/app/pipeline/`: `aspect.rs`, `camera.rs`, `paint.rs`, `profile.rs`, `mod.rs`.

3. **Zero-Allocation UI & Render Loop Rules**:
   - In immediate-mode UI loops, prefer zero-allocation tuple salts `("salt", id)` over heap-allocating `format!(...)`.
   - Never allocate `String`s or dynamic `Vec`s per frame for tick labels; use stack buffers (`[u8; 32]`) and stack arrays (e.g. `[TickMark; 7]`).
   - Use zero-allocation case-insensitive ASCII searches (`contains_ascii_case_insensitive`) from `crate::data::coordinates::naming` for dimension and coordinate parsing.
   - All custom canvas overlays and floating toolbars must dynamically adapt to dark and light visual themes (`ui.visuals().dark_mode`).
   - Transient UI overlays (crop handles, grids, tooltips) must be suppressed during export capture passes (`if self.pending_export.is_none()`).
   - **No raw emojis or unicode glyphs**: Do not use font-dependent emojis or unicode symbols in UI widgets, buttons, labels, logs, or status messages. Use procedural vector icons from `crate::ui::icons::Icon` via `UiIconExt` (`icon`, `icon_toned`, `icon_button`, `icon_label`), `ToolbarButton` for frameless toolbar buttons, or `Icon::paint` for decorative art. Size icons only with `IconSize` (`Xs` 12, `Sm` 14, `Md` 18, `Lg` 24) and color them with theme-adaptive `IconTone`s instead of hard-coded `Color32`s. Close and clear-field buttons always use `ui.close_button(hover)` (frameless `Icon::Cross`, no label); close buttons sit in the top-right corner of their panel or window, in place of egui's built-in title-bar close. If a new visual symbol is needed, define it as a procedural vector icon in `src/ui/icons/` with dark/light theme support.

4. **Skills Reference**:
   - `rust-skills`: 265 detailed Rust best practices across 26 categories (install locally via `git clone --depth 1 https://github.com/leonardomso/rust-skills.git .agents/skills/rust-skills && rm -rf .agents/skills/rust-skills/.git`).
   - `rust-workflows`: Cargo build, test, lint, clippy, WASM, and logging commands.
   - `octant-data-engine`: Data loading, caching, prefetching, and hyperslab slicing.
   - `octant-rendering-wgpu`: WGPU render pipelines, uniform buffer alignment, and WGSL shaders.
   - `octant-ui-egui`: egui immediate-mode GUI components and event dispatch.
