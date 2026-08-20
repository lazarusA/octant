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
    BlockPrefetcher (background thread pool lookahead)
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

## Key Components

### 1. `BlockStore` Trait (`src/data/block_store.rs`)
To add a new storage backend:
- Implement `backend_name(&self) -> &'static str`
- Implement `variables(&self) -> &[VariableInfo]`
- Implement `inspect(&self) -> &DatasetMetadata`
- Implement `fetch_block(&self, request: &BlockRequest) -> Result<OctantBlock, DataError>`
- Implement `fetch_blocks(&self, requests: &[BlockRequest]) -> Result<Vec<OctantBlock>, DataError>`

### 2. `OctantBlock` (`src/data/octant_block.rs`)
- Represents an in-memory resident hyperslab of rank $N$ with row-major strides.
- `slice_2d(dim_x, dim_y, fixed_indices) -> Result<MatrixData, DataError>` extracts 2D matrices for GPU rendering.
- `volume(dim_x, dim_y, dim_z, fixed_indices) -> Result<MatrixData, DataError>` extracts 3D volumetric slabs.
- Ensure all numeric values are cast/normalized to `f32` without unnecessary intermediate allocations (`mem-zero-copy`, `anti-collect-intermediate`).

### 3. `BlockCache` & `BlockPrefetcher` (`src/data/block_cache.rs`, `src/data/block_prefetch.rs`)
- `BlockCacheKey` hashes dataset URI, variable name, and selection ranges.
- `BlockPrefetcher` runs a dedicated background worker pool to fetch upcoming frames along the time/animation dimension before the UI requests them.
- Always use non-blocking channel polling (`try_recv`) in the main UI thread.

### 4. Async vs Sync Execution
- Desktop targets use `tokio` multi-threaded runtime for parallel I/O.
- WASM targets require `wasm-bindgen-futures`. Avoid blocking threads on WASM.
