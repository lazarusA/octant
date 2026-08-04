# octant

A cloud-native visualization engine in Rust for n-dimensional datasets and interactive plotting.

## Features

- **Flexible Data Storage**: Seamless access to local filesystems and remote cloud object stores (S3, HTTP, Azure, GCP) via `object_store`, [Zarr](https://zarr.dev/), and [Icechunk](https://icechunk.io/).
- **Versatile Plotting & Inspection**: Interactive n-dimensional data exploration with diverse plot types, 1D/2D slice renderers, and spatial visualizers.
- **Hardware-Accelerated Rendering**: Fast graphics pipeline powered by `wgpu` and `egui` / `eframe`.
- **Cross-Platform & Web**: Native desktop application with full WebAssembly (WASM) browser support [WIP].

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
