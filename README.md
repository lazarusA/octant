# octant

An interactive viewer for n-dimensional datasets with native support for local and cloud object storage, Zarr, and Icechunk. Built in Rust with GPU-accelerated rendering.

## Features

- **Flexible Data Storage**: Seamless access to local filesystems and remote cloud object stores (S3, HTTP, Azure, GCP) via `object_store`, [Zarr](https://zarr.dev/), and [Icechunk](https://icechunk.io/).
- **Interactive Plotting**: Explore n-dimensional data with diverse plot types, 1D/2D slice renderers, and 3D spatial visualizers.
- **Hardware-Accelerated Rendering**: Fast rendering powered by `wgpu` and `egui` / `eframe`.
- **Cross-Platform**: Native desktop application with full WebAssembly (WASM) browser support [WIP].

## Install

```bash
cargo install octant
```

Running the above command will globally install the **octant** binary.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE)
- MIT License [LICENSE-MIT](LICENSE-MIT)

at your option.
