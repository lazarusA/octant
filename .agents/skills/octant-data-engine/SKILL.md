---
name: octant-data-engine
description: >-
  Deep domain knowledge for Octant's N-dimensional data system, storage backends
  (Zarr, Icechunk), OctantBlock slicing, LRU caching (BlockCache), and background
  prefetching (BlockPrefetcher). Use when modifying or debugging src/data/.
---

# Octant Data Engine Skill

This skill guides development and debugging of Octant's N-dimensional tensor extraction, caching, and storage backends in `src/data/`.

## Architecture Overview

```
Storage Backend (Zarr/Icechunk)
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
    OctantBlock (N-D strided memory block)
            │
            ▼
    MatrixData / Volume (f32 renderable GPU payload)
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

### 2. `OctantBlock` (`src/data/octant_block.rs`)
- Represents an in-memory resident hyperslab of rank $N$ with row-major strides.
- Store values in `Arc<[f32]>` for $O(1)$ zero-copy sharing between cache, resamplers, and GPU upload buffers.
- `slice_2d(dim_x, dim_y, fixed_indices) -> Result<MatrixData, DataError>` extracts 2D matrices for GPU rendering.
- `volume(dim_x, dim_y, dim_z, fixed_indices) -> Result<MatrixData, DataError>` extracts 3D volumetric slabs.
- When validating tensor volume/length, always use checked arithmetic:
  ```rust
  shape.iter().copied().try_fold(1usize, |acc, d| acc.checked_mul(d)).unwrap_or(usize::MAX)
  ```

### 3. `BlockCache` & `BlockPrefetcher` (`src/data/block_cache.rs`, `src/data/block_prefetch.rs`)
- `BlockCacheKey` hashes dataset URI, variable name, and selection ranges.
- `BlockPrefetcher` runs a dedicated worker pool to fetch upcoming frames along the animation dimension before the UI requests them.
- Always use **bounded channels** (`sync_channel(N)`) to ensure backpressure when the UI consumer lags behind prefetch workers.
- Always poll channels with non-blocking `try_recv()` inside the main UI loop.

### 4. Async vs Sync Execution
- Desktop targets use `tokio` multi-threaded runtime for parallel I/O.
- WASM targets require `wasm-bindgen-futures`. Avoid blocking threads on WASM.
