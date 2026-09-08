---
name: octant-data-engine
description: >-
  Deep domain knowledge for Octant's N-dimensional data system, storage backends
  (Zarr, Icechunk), OctantBlock slicing, LRU caching (BlockCache), and background
  prefetching (BlockPrefetcher). Use when modifying or debugging src/data/.
---

# Octant Data Engine Skill

This skill guides development and debugging of Octant's N-dimensional tensor extraction, caching, storage backends, and hyperslab slicing in `src/data/`.

## Architecture Overview

```
Storage Backend (Zarr/Icechunk/NetCDF)
            │
            ▼
    BlockStore Trait (inspect, fetch_block, fetch_blocks)
            │
            ▼
    DatasetManager (StoreHandle registry & lifecycle)
            │
            ▼
    BlockPrefetcher (bounded background lookahead pool)
            │
            ▼
    BlockCache (LRU cache of OctantBlock hyperslabs)
            │
            ▼
    OctantBlock (N-D strided memory block with Arc<[f32]>)
            │
            ▼
    Hyperslab Slicing Subsystem (src/data/slicing/)
    ├── common.rs   (range clamping & fixed dimension offsets)
    ├── coords.rs   (sliced coordinate extraction)
    ├── slice_2d.rs (fast-path 2D/1D/0D slices)
    └── slice_3d.rs (fast-path 3D volumetric slabs)
            │
            ▼
    MatrixData / VolumeData (f32 renderable GPU payload)
```

## Key Invariants & Best Practices

### 1. `BlockStore` Trait (`src/data/block_store.rs`)
To add a new storage backend:
- Implement `backend_name(&self) -> &'static str`
- Implement `variables(&self) -> &[VariableInfo]`
- Implement `inspect(&self) -> &DatasetMetadata`
- Implement `fetch_block(&self, request: &BlockRequest) -> Result<OctantBlock, DataError>`
- Implement `fetch_blocks(&self, requests: &[BlockRequest]) -> Result<Vec<OctantBlock>, DataError>`
- Remote storage clients (`object_store::ClientOptions`) **must always configure explicit timeouts** (`.with_timeout(Duration::from_secs(30))` and `.with_connect_timeout(Duration::from_secs(10))`) to prevent worker starvation.

### 2. `OctantBlock` & Zero-Copy Tensor Sharing (`src/data/octant_block.rs`)
- Represents an in-memory resident hyperslab of rank $N$ with row-major strides.
- Store values in `Arc<[f32]>` for $O(1)$ zero-copy sharing between cache, prefetcher, resamplers, and GPU upload buffers.
- `slice_2d(...) -> Option<MatrixData>` extracts 2D matrices for GPU rendering.
- `volume(...) -> Option<VolumeData>` extracts 3D volumetric slabs.
- When computing tensor volume/length, always use checked arithmetic:
  ```rust
  shape.iter().copied().try_fold(1usize, |acc, d| acc.checked_mul(d)).unwrap_or(usize::MAX)
  ```

### 3. Hyperslab Slicing Subsystem (`src/data/slicing/`)
- **Mathematical Invariants (`common.rs`)**:
  - Use `clamp_slice_range(range, full_len) -> (start, end, span)` to prevent out-of-bounds indexing.
  - Use `compute_fixed_dims_offset(...)` to compute linear base offsets across pinned non-spatial dimensions.
  - Use `resolve_min_max(compute_bounds, values, block_min, block_max)` to avoid redundant $O(N)$ min/max scans when pre-computed block bounds exist.
- **2D Fast-Path Slicing (`slice_2d.rs`)**:
  - `copy_contiguous_slice`: Full slice copied in a single contiguous chunk.
  - `copy_row_contiguous_slice`: Fast path when `stride_x == 1`, copying entire rows with `extend_from_slice`.
  - `copy_strided_slice`: General strided fallback.
- **3D Volumetric Slicing (`slice_3d.rs`)**:
  - `is_full_contiguous_3d`: Direct zero-copy slice for fully resident 3D tensors.
  - `clamp_gpu_volume_depth(nx, ny, nz)`: Automatically clamps effective depth $Z$ so the voxel volume never exceeds `MAX_GPU_STORAGE_BUFFER_ELEMENTS` (128 MiB default limit).

### 4. `BlockCache` & `BlockPrefetcher` (`src/data/block_cache.rs`, `src/data/block_prefetch.rs`)
- `BlockCacheKey` hashes dataset URI, variable name, and selection ranges.
- `BlockPrefetcher` runs a dedicated worker pool to fetch upcoming frames along the animation dimension before the UI requests them.
- Always use **bounded channels** (`sync_channel(N)`) to ensure backpressure when the UI consumer lags behind prefetch workers.
- Always poll channels with non-blocking `try_recv()` inside the main UI loop.

### 5. Async vs Sync Execution
- Desktop targets use `tokio` multi-threaded runtime for parallel I/O.
- WASM targets require `wasm-bindgen-futures`. Avoid blocking threads on WASM.
