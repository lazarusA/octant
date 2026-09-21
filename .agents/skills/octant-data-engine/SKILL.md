---
name: octant-data-engine
description: >-
  Deep domain knowledge for Octant's N-dimensional data system, storage backends
  (Zarr, Icechunk, NetCDF, Procedural), codecs, coordinates, OctantBlock slicing,
  LRU caching (BlockCache), and background prefetching (BlockPrefetcher). Use when
  modifying or debugging src/data/.
---

# Octant Data Engine Skill

This skill guides development and debugging of Octant's N-dimensional tensor extraction, caching, storage backends, codecs, and hyperslab slicing in `src/data/`.

## Architecture Overview

```
Storage Backends (src/data/backends/)
├── http/           (Browser window.fetch / Desktop reqwest range requests)
├── coord_bounds/   (Unified coordinate bounds extraction & global caching)
├── zarr/           (Zarr v2/v3, chunk cache, async wasm HTTP streaming)
├── icechunk/       (Native iceberg-style repository & pure-Rust wasm client)
├── netcdf/         (Desktop libnetcdf FFI & wasm notice handlers)
└── procedural/     (Analytical 2D/3D/4D grids, Gaussians, HEALPix synthesis)
            │
            ▼
Codec Normalization & Plugins (src/data/codecs/)
├── blusc_plugin.rs (Pure Rust Blosc codec plugin for zarrs)
├── zstd_plugin.rs  (Pure Rust Zstd streaming codec plugin for zarrs)
└── normalize.rs    (Zarr v3 metadata normalization & codec reordering)
            │
            ▼
Coordinates Subsystem (src/data/coordinates/)
├── naming.rs       (Dimension role inference & zero-allocation ASCII search)
├── detection.rs    (Grid regularity, step sizing & coordinate spans)
├── healpix/        (Ring & Nested HEALPix indexing and pixel boundaries)
├── dggs.rs         (Discrete Global Grid Systems metadata & decoding)
└── topology.rs     (1D/2D spatial and temporal coordinate topologies)
            │
            ▼
Blocks Engine (src/data/blocks/)
├── store.rs        (BlockStore trait: backend_name, variables, inspect, fetch_block)
├── loader.rs       (Async block loader workers for native & wasm runtimes)
├── prefetch.rs     (BlockPrefetcher lookahead pool over bounded channels)
├── cache.rs        (LRU memory cache of resident OctantBlocks)
└── key.rs          (BlockKey hashing URI, variable, and hyperslab ranges)
            │
            ▼
OctantBlock (src/data/octant_block.rs)
(N-D resident memory block with zero-copy Arc<[f32]> tensor storage)
            │
            ▼
Hyperslab Slicing Subsystem (src/data/slicing/)
├── common.rs       (Range clamping, fixed dimension linear offsets)
├── coords.rs       (Sliced coordinate bounds & coordinate array extraction)
├── copy.rs         (Strided & contiguous buffer copy routines)
├── slice_1d.rs     (1D & 0D scalar profile slices)
├── slice_2d.rs     (Fast-path 2D planar matrix slices)
└── slice_3d.rs     (Fast-path 3D volumetric slabs with GPU depth clamping)
            │
            ▼
MatrixData / VolumeData (f32 renderable GPU payload in src/data/matrix_data.rs)
```

## Subsystem Breakdown & Directory Rules

### 1. Storage Backends (`src/data/backends/`)
Every backend submodule is strictly modularized into single-responsibility files (< 250 LOC per file):
- **`geotiff/`**:
  - `reader.rs`: Async Cloud-Optimized GeoTIFF (COG) / TIFF header reader, IFD directory discovery, and pooled HTTP range fetching.
  - `slice.rs`: GeoTIFF tile hyperslab extraction, RGB/single-band rendering buffer extraction, and coordinate bounding box calculation.
  - `tests.rs`: COG header parsing, tile index arithmetic, and coordinate spatial bounds tests.
  - `mod.rs`: `GeoTiffBlockStore` implementation of the `BlockStore` trait.
- **`http/`**:
  - `fetch.rs`: Low-level byte fetching and HTTP Range requests. WASM targets use browser `window.fetch()` with `js_sys::Uint8Array`; Desktop targets use a static `OnceLock<reqwest::Client>` configured with explicit connect (10s) and request (30s) timeouts.
- **`coord_bounds/`**:
  - `cache.rs`: Global thread-safe `COORD_VALUES_CACHE` for coordinate string vectors and numerical bounds.
  - `candidates.rs`: Coordinate variable discovery and standard alias matching (`lat`, `lon`, `time`, `depth`).
  - `discover.rs`: Group-aware array resolution across nested dataset hierarchies.
  - `extract.rs`: Multi-dimensional coordinate range extraction and boundary slicing.
- **`zarr/`**:
  - `generic.rs`: `GenericZarrBlockStore` over `ReadableWritableListableStorage` with per-array LRU chunk caching (`ChunkCacheDecodedLruSizeLimit`).
  - `store.rs`: Concrete desktop/native `ZarrBlockStore`.
  - `block.rs`: Hyperslab extraction, coordinate orientation, and attribute inheritance.
  - `slice.rs`: Subset reading, type casting to `f32`, and fill-value masking.
  - `storage.rs`: Synchronous storage adapter construction (`object_store::http` and `FilesystemStore`).
  - `zstd_shim.rs`: Pure-Rust `extern "C"` shims (`ZSTD_*`) backed by `ruzstd` for WASM linking.
  - `wasm/`: Async browser streaming (`inspect.rs`, `loader.rs`, `preload.rs`, `store.rs`).
- **`icechunk/`**:
  - `native.rs`: Desktop `IcechunkBlockStore` connecting to S3 / local repositories via `icechunk` crate.
  - `wasm/`: Pure-Rust client (`discovery.rs`, `header.rs`, `inspect.rs`, `loader.rs`, `preload.rs`, `store.rs`) resolving manifests over HTTP.
- **`netcdf/`**:
  - `desktop.rs`: Native NetCDF4 / HDF5 file reader using `libnetcdf` FFI.
  - `inspect.rs`, `slice.rs`, `attrs.rs`, `coords.rs`, `wasm.rs`.
- **`procedural/`**:
  - `store.rs`: Synthetic procedural datasets (Wave Packets, Gaussian Spheres, Stepped Resolutions, Regional Grids).
  - `slice_2d.rs`, `slice_3d.rs`, `healpix.rs`, `healpix_meta.rs`, `inspect.rs`.

### 2. Codecs Subsystem (`src/data/codecs/`)
- `blusc_plugin.rs`: Pure Rust Blosc compressor & decompressor plugin registered via `zarrs_codec` and `inventory`.
- `zstd_plugin.rs`: Pure Rust Zstandard streaming decoder plugin using `ruzstd`.
- `normalize.rs`: Normalizes Zarr v3 codec pipelines (e.g. ensuring `bytes` codec precedes `blosc`/`zstd` compression).

### 3. Coordinates Subsystem (`src/data/coordinates/`)
- `naming.rs`: Case-insensitive ASCII search (`contains_ascii_case_insensitive`) and dimension role detection (`is_spatial_x_name`, `is_spatial_y_name`, `is_spatial_z_name`, `is_animated_time_name`).
- `detection.rs`: Coordinate spacing uniformity, ascending/descending orientation, and regional boundary detection.
- `healpix/`: HEALPix grid geometry, ring/nested index calculations, and resolution schemes.
- `dggs.rs`: Discrete Global Grid Systems parsing.

### 4. Blocks Engine (`src/data/blocks/`)
- `store.rs`: The polymorphic `BlockStore` trait:
  ```rust
  pub trait BlockStore: Send + Sync {
      fn backend_name(&self) -> &str;
      fn variables(&self) -> Result<Vec<String>, BlockStoreError>;
      fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError>;
      fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError>;
      fn fetch_block_with_progress(&self, request: &SliceRequest, on_progress: ProgressCallback) -> Result<OctantBlock, BlockStoreError>;
      fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError>;
  }
  ```
- `cache.rs`: Memory-bounded LRU cache storing `OctantBlock`s.
- `prefetch.rs`: Bounded background prefetching along animation dimensions using `sync_channel`.

### 5. Data Slicing Subsystem (`src/data/slicing/`)
- `common.rs`: Zero-allocation math, range clamping (`clamp_slice_range`), and linear base offset computation.
- `slice_2d.rs`: Fast-path 2D planar extraction (contiguous, row-contiguous, strided).
- `slice_3d.rs`: Fast-path 3D volumetric slabs with automatic GPU storage buffer limit clamping (`clamp_gpu_volume_depth`).

## Key Invariants & Best Practices

1. **Zero-Copy Tensor Memory (`Arc<[f32]>`)**:
   - Store all block values in `Arc<[f32]>` for $O(1)$ sharing between cache, resamplers, and GPU render pipelines.
2. **Checked Arithmetic on Tensor Dimensions**:
   - Always use checked operations when multiplying tensor shapes:
     ```rust
     shape.iter().copied().try_fold(1usize, |acc, d| acc.checked_mul(d)).unwrap_or(usize::MAX)
     ```
3. **Poison-Resilient Locks**:
   - Always handle `RwLock` and `Mutex` guards with `.unwrap_or_else(|p| p.into_inner())` or `if let Ok(guard) = ...`.
4. **No Raw Emojis or Unicode Symbols**:
   - All visual icons must come from `crate::ui::icons::Icon`.
5. **No `unwrap()` in Production Code**:
   - Use `?`, `let Some(...) = ... else`, or `f32::total_cmp`.
