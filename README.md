# octant

[<img alt="github" src="https://img.shields.io/badge/github-lazarusA/octant-8da0cb?logo=github" height="20">](https://github.com/lazarusA/octant)
[![Latest version](https://img.shields.io/crates/v/octant.svg)](https://crates.io/crates/octant)
[![Build Status](https://github.com/lazarusA/octant/workflows/Rust/badge.svg)](https://github.com/lazarusA/octant/actions/workflows/rust.yml)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/lazarusA/octant/blob/main/LICENSE-MIT)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/lazarusA/octant/blob/main/LICENSE-APACHE)

An interactive viewer for n-dimensional datasets with native support for local and cloud object storage, Zarr, and Icechunk. Built in Rust with GPU-accelerated rendering.

## Features

- **Flexible Data Storage**: Seamless access to local filesystems and remote cloud object stores (S3, HTTP, Azure, GCP) via `object_store`, [Zarr](https://zarr.dev/), and [Icechunk](https://icechunk.io/).
- **Interactive Plotting**: Explore n-dimensional data with diverse plot types, 1D/2D slice renderers, and 3D spatial visualizers.
- **Hardware-Accelerated Rendering**: Fast rendering powered by `wgpu` and `octant` / `eframe`.
- **Cross-Platform**: Native desktop application with full WebAssembly (WASM) browser support [WIP].

## Install

```bash
cargo install octant
```

Running the above command will globally install the **octant** binary.

## WebAssembly Build & Local Preview

To build and preview Octant in your web browser:

1. **Install Trunk** (and WASM target if needed):
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install trunk
   ```

2. **Run Local Development Server (Live Reload & Preview)**:
   ```bash
   trunk serve
   ```
   Open `http://127.0.0.1:8080` in your browser.

3. **Build Production Static Release**:
   ```bash
   trunk build --release
   ```
   The compiled deployable output will be generated in the `dist/` directory, ready to be hosted on any web server or static host like GitHub Pages.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE)
- MIT License [LICENSE-MIT](LICENSE-MIT)

at your option.
