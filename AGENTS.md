# Octant Agent Rules

When developing and reviewing code in this repository:

1. **Rust Quality & Checks**:
   - Adhere to idiomatic Rust (Rust 2024 edition).
   - Enforce borrow-over-clone (`&[T]` over `&Vec<T>`, `&str` over `&String`).
   - Forbid `unwrap()` in production code (use `?`, `let Some(...) = ... else`, or `f32::total_cmp`).
   - Use poison-resilient lock handling (`if let Ok(guard) = ...` or `.unwrap_or_else(|p| p.into_inner())`).
   - Always use checked arithmetic when computing multi-dimensional tensor shape volumes (`shape.iter().try_fold(...)`).
   - Run `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`.

2. **Octant Architecture**:
   - Tensor data loading lives in `src/data/` (Zarr, Icechunk, `OctantBlock`, `BlockCache`, `BlockPrefetcher`).
   - Remote I/O must specify client timeouts (`with_timeout(30s)`, `with_connect_timeout(10s)`).
   - Background prefetchers and loaders must use bounded channels (`sync_channel`) for backpressure.
   - WGPU renderers and WGSL shaders live in `src/plots/` and `src/plots/shaders/` (embedded via `assemble_plot_shader!`).
   - For all structured, discrete global (HEALPix, ICON, Cubed-Sphere), and unstructured grid formats (UGRID, MPAS), use zero-allocation GPU instancing or GPU vertex pulling. Reserve CPU mesh generation strictly for non-grid geometry (GIS vector polygons, streamlines, Marching Cubes CAD export, UI labels and annotations).
   - UI widgets and layout live in `src/ui/` and `src/app/`, with state actions routed via `AppAction`.
   - In immediate-mode UI loops, prefer zero-allocation tuple salts `("salt", id)` over heap-allocating `format!(...)`.
   - **Canvas Capture & Animation Export**:
     - Single image saves (`.png`) must embed Apple Display P3 color profile chunks (`cHRM` with D65 white point + `gAMA` 2.2) via `src/utils/png.rs` to ensure accurate wide-gamut rendering on Retina displays.
     - Video exports (`.mp4`) must use ITU-R BT.709 Full-Range YUV420 encoding with 30 Mbps bitrate (`src/utils/video.rs`) to prevent washed-out colors.
     - Live interactive recording uses a 3-buffer non-blocking staging ring (`CaptureRing` in `src/app/capture_ring.rs`) to avoid GPU fence stalls.
     - Animation export architecture supports deterministic motion trajectories (`MotionTrajectory`, `ZoomTrajectory`) with automatic camera state restoration upon completion or cancellation.

3. **Skills Reference**:
   - `rust-skills`: 265 detailed Rust best practices across 26 categories (ownership, error handling, memory, async, unsafe, etc.).
   - `rust-workflows`: Cargo build, test, lint, clippy, WASM, and logging commands.
   - `octant-data-engine`: Data loading, caching, prefetching, and hyperslab slicing.
   - `octant-rendering-wgpu`: WGPU render pipelines, uniform buffer alignment, WGSL shaders, and offscreen targets.
   - `octant-ui-egui`: egui immediate-mode GUI components, color spaces, export workflows, and event dispatch.

