# Octant Agent Rules

When developing and reviewing code in this repository:

1. **Rust Quality & Checks**:
   - Adhere to idiomatic Rust (Rust 2024 edition).
   - Enforce borrow-over-clone (`&[T]` over `&Vec<T>`, `&str` over `&String`).
   - Forbid `unwrap()` in production code.
   - Run `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`.

2. **Octant Architecture**:
   - Tensor data loading lives in `src/data/` (Zarr, Icechunk, `OctantBlock`, `BlockCache`, `BlockPrefetcher`).
   - WGPU renderers and WGSL shaders live in `src/plots/` and `assets/shaders/`.
   - UI widgets and layout live in `src/ui/` and `src/app/`, with state actions routed via `AppAction`.

3. **Skills Reference**:
   - `rust-skills`: 265 detailed Rust best practices across 26 categories (ownership, error handling, memory, async, unsafe, etc.).
   - `rust-workflows`: Cargo build, test, lint, clippy, WASM, and logging commands.
   - `octant-data-engine`: Data loading, caching, prefetching, and hyperslab slicing.
   - `octant-rendering-wgpu`: WGPU render pipelines, uniform buffer alignment, and WGSL shaders.
   - `octant-ui-egui`: egui immediate-mode GUI components and event dispatch.
