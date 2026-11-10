# Octant Agent Rules

When developing and reviewing code in this repository:

1. **Rust Quality & Checks**:
   - Adhere to idiomatic Rust (Rust 2024 edition).
   - Enforce borrow-over-clone (`&[T]` over `&Vec<T>`, `&str` over `&String`).
   - Forbid `unwrap()` and `expect()` in production code (use `?`, `let Some(...) = ... else`, or `f32::total_cmp`). The one exception is `expect()` on a value the app cannot run without and nothing could recover from, such as building the shared Tokio runtime at startup (`utils/executor.rs`); never on data, files, the network or user input.
   - Use poison-resilient lock handling (`if let Ok(guard) = ...` or `.unwrap_or_else(|p| p.into_inner())`).
   - Always use checked arithmetic when computing multi-dimensional tensor shape volumes (`shape.iter().try_fold(...)`).
   - Run `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`.
   - Development tools kept though nothing calls them: `utils/diagnostics.rs` (`log_gpu_diagnostics`, `log_lod_tile_upload`, called by hand while debugging). Dead-code audits leave it alone.
   - Future work kept though nothing uses it yet: `utils/error.rs` `OctantError`, the typed error meant to replace `BlockStoreError` (`Box<dyn Error>`) and `Result<_, String>`; adopt it backend by backend, starting where callers need to tell failures apart.
   - For C-library/FFI test suites (e.g. NetCDF), serialize file creation with static mutex test locks to prevent concurrent non-reentrant IO collisions.
   - Release profile, `wasm-opt` flags and the CI's wasm size budgets: `.agents/skills/rust-workflows/build.md`; raise a budget only in a dedicated commit that explains the growth.

2. **Modular Architecture & Subsystem Layout**:
   - Follow the Open-Closed Principle (OCP): decompose monolithic modules into single-purpose submodules (< 250 lines per file, < 50 lines per function).
   - Each subsystem's files, types and behaviour are described in the skills under `.agents/skills/` (section 5). Read the reference file for the code you touch, and update it with the code.
   - Rules from them that most tasks need:
     - Store tensor values in `Arc<[f32]>` for $O(1)$ zero-copy sharing between cache and render pipelines.
     - Code indexing a block in stored order goes through `src/data/block_orientation.rs` (`oriented_range`, `slab_destination`).
     - Coordinates may be missing until they arrive: read them through `get_dim_coords` and fall back to indices. Read coordinates by index (`get`, `number_for`, `range_bounds`); never iterate them in per-frame UI code.
     - Caches of data derived from loaded metadata key on `VariableSelection::metadata_generation` and `coordinates_revision`, never on the metadata's address.
     - To add a reader: one `DataSourceKind` variant per format (no catch-all), matched in `dataset/factory.rs`, `app/state/store_kind.rs`, `ui/dataset/manager.rs` `source_kind_icon`, `utils/path.rs` and the browser's `backends/web.rs` `WebFormat::of`.
     - Never hand-write colormap formulas in Rust or WGSL: add maps to the catalog or registry instead. Tests that add or remove custom maps or depend on row ids take `registry::test_lock()`.
     - Never project in a shader or the hover by hand; read the camera (`src/plots/camera.rs`).
     - Uniform parameter structs (such as `Mesh3DUniformParams`) must be zero-allocation `Copy` structs; never clone large coordinate vectors or data structures inside paint loops.
     - GPU uniform structs are `#[repr(C)]`, `bytemuck::Pod`/`Zeroable`, laid out by WGSL's std140/std430 rules (`vec4` 16-byte aligned) with explicit padding fields; `shader_tests.rs` `uniform_layouts_match_wgsl` checks them.
     - For all structured, discrete global (HEALPix, ICON, Cubed-Sphere), and unstructured grid formats (UGRID, MPAS), use zero-allocation GPU instancing or GPU vertex pulling. Reserve CPU mesh generation strictly for non-grid geometry (GIS vector polygons, streamlines, Marching Cubes CAD export, UI labels and annotations).
     - Read colors from `ui.visuals()`, never hard-code theme grays; floating panels keep the theme border (`Frame::popup`/`window`, no `Stroke::NONE`) so overlapping panels stay distinguishable.
     - After changing a CJK translation run `cargo run --features font-tools --bin subset_fonts` (`.agents/skills/octant-ui-egui/i18n.md`).

3. **Zero-Allocation UI & Render Loop Rules**:
   - In immediate-mode UI loops, prefer zero-allocation tuple salts `("salt", id)` over heap-allocating `format!(...)`.
   - Never format tick labels or build dynamic `Vec`s per frame: format into stack buffers (`[u8; 32]`, `stack_str`) and stack arrays (e.g. `[TickMark; 7]`), or cache formatted labels until their inputs change (`ui::temp_cache::cached`). egui's own copy of the text when laying it out (`painter.text`, `layout_no_wrap`) is unavoidable and fine.
   - Never keep text galleys (`Arc<Galley>`) across frames: egui rebuilds its fonts and glyph atlas when the text options change (switching dark/light mode) or the atlas fills up, and an old galley then draws stale atlas texels as scrambled text. Lay text out every frame through egui (its galley cache makes unchanged text a hash lookup); within one frame, laying out once and reusing the galley is fine.
   - Use zero-allocation case-insensitive ASCII searches (`contains_ascii_case_insensitive`) from `crate::data::coordinates::naming` for dimension and coordinate parsing.
   - All custom canvas overlays and floating toolbars must dynamically adapt to dark and light visual themes (`ui.visuals().dark_mode`).
   - Transient UI overlays (crop handles, grids, tooltips) must be suppressed during export capture passes (`if app.pending_export.is_none()`).
   - User-facing failures, warnings and results go through `OctantApp::notify` / `push_notice` (with the app) or `crate::ui::toast::report` (renderers, backends, background threads), which show a canvas toast and log it; never only the log. Keep internal fallbacks and per-chunk failures that already surface as one higher-level error log-only.
   - All user-facing text goes through `crate::i18n::tr` (text with values through `i18n::fill`/`filled` templates); add the key to its area in English and in every language file. Give egui widgets whose label is translated a fixed `id_salt` so their state survives a language switch.
   - **No raw emojis or unicode glyphs**: Do not use font-dependent emojis or unicode symbols in UI widgets, buttons, labels, logs, or status messages. Use procedural vector icons from `crate::ui::icons::Icon` via `UiIconExt` (`icon`, `icon_toned`, `icon_colored`, `icon_button`), `ToolbarButton` for frameless toolbar buttons, or `Icon::paint` for decorative art. Size icons only with `IconSize` (`Xs` 12, `Sm` 14, `Md` 18, `Lg` 24) and color them with theme-adaptive `IconTone`s instead of hard-coded `Color32`s. Close and clear-field buttons always use `ui.close_button(hover)` (frameless `Icon::Cross`, no label); close buttons sit in the top-right corner of their panel or window, in place of egui's built-in title-bar close. If a new visual symbol is needed, define it as a procedural vector icon in `src/ui/icons/` with dark/light theme support.

4. **Licensing & Redistribution**:
   - Regenerate `assets/licenses/THIRD_PARTY_LICENSES.md` (`scripts/third_party_licenses.sh`) whenever `Cargo.lock` changes; CI fails when it is stale. Clarified licenses, packaging and where the notices ship: `.agents/skills/rust-workflows/build.md`.

5. **Skills** (`.agents/skills/`, each with its reference files):
   - `octant-data-engine`: `src/data/`; `data.md` (blocks, prefetch, data workers, backends, OME-Zarr, CRS, multiscale, codecs, coordinates, slicing, orientation), `catalog.md` (`src/catalog/`).
   - `octant-rendering-wgpu`: `src/plots/`; `plots.md` (renderers, volume raymarcher, OIT, camera, slicer shaders, buffer limits), `colormaps.md` (`src/utils/colormap/`), `export.md` (`src/export/`).
   - `octant-ui-egui`: `src/ui/` and `src/app/`; `ui.md`, `app.md` (state, actions, share links, controllers, canvas, pipeline, layers, block loading), `i18n.md` (translations and fonts).
   - `rust-workflows`: build, test, lint, clippy and WASM commands; `build.md` (release profile, size budgets, licensing).
   - `rust-skills`: 265 detailed Rust best practices across 26 categories (install locally via `git clone --depth 1 https://github.com/leonardomso/rust-skills.git .agents/skills/rust-skills && rm -rf .agents/skills/rust-skills/.git`).
