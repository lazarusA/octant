<h1 align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/lazarusA/octant/17489ba8410364d3fd0d11c697eb809f990e3486/assets/brand/svg/octant-logo-dark.svg"><img alt="" src="https://raw.githubusercontent.com/lazarusA/octant/17489ba8410364d3fd0d11c697eb809f990e3486/assets/brand/svg/octant-logo-light.svg" height="40" align="center"></picture> octant</h1>

<p align="center">
  <a href="https://github.com/lazarusA/octant"><img alt="github" src="https://img.shields.io/badge/github-lazarusA/octant-8da0cb?logo=github"></a>
  <a href="https://lazarusa.github.io/octant/"><img alt="Live Demo" src="https://img.shields.io/badge/demo-online%20(wasm)-blue?logo=webassembly&logoColor=white"></a>
  <a href="https://crates.io/crates/octant"><img alt="Latest version" src="https://img.shields.io/crates/v/octant.svg"></a>
  <a href="https://github.com/lazarusA/octant/actions/workflows/rust.yml"><img alt="Build Status" src="https://github.com/lazarusA/octant/actions/workflows/rust.yml/badge.svg"></a>
  <a href="https://github.com/lazarusA/octant/blob/main/LICENSE-MIT"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <a href="https://github.com/lazarusA/octant/blob/main/LICENSE-APACHE"><img alt="Apache" src="https://img.shields.io/badge/license-Apache-blue.svg"></a>
</p>

<p align="center">
  <a href="https://github.com/lazarusA/octant/releases/latest/download/Octant-macOS-arm64.dmg"><img alt="Download for macOS (Apple silicon)" src="https://img.shields.io/badge/macOS-Apple%20silicon-3a3f44?style=for-the-badge&labelColor=121416"></a>
  <a href="https://github.com/lazarusA/octant/releases/latest/download/Octant-Windows-x64-setup.exe"><img alt="Download for Windows (x64)" src="https://img.shields.io/badge/Windows-x64-3a3f44?style=for-the-badge&labelColor=121416"></a>
  <a href="https://github.com/lazarusA/octant/releases/latest/download/Octant-Linux-x86_64.AppImage"><img alt="Download for Linux (x86_64 AppImage)" src="https://img.shields.io/badge/Linux-AppImage-3a3f44?style=for-the-badge&labelColor=121416"></a>
</p>

An interactive viewer for N-dimensional datasets, stored locally or in cloud object storage. Built in Rust with GPU rendering, available on desktop and in the browser.

![Octant Screenshot](https://raw.githubusercontent.com/lazarusA/octant/1f9797486adcc7d2ea43409d2d248771d90a4d33/assets/octant_tas.png)

**[Try it in your browser](https://lazarusa.github.io/octant/)**: nothing to install (WebGPU, with a WebGL2 fallback).

## Install

<details>
<summary>Every installer, and how to install it</summary>

Each [release](https://github.com/lazarusA/octant/releases/latest) holds:

| System | File | Install |
|---|---|---|
| macOS (Apple silicon) | `Octant-macOS-arm64.dmg` | Open it and drag Octant to Applications |
| macOS (Apple silicon) | `Octant_<version>_aarch64.app.zip` | Unzip it and move `Octant.app` to Applications |
| Windows (x64) | `Octant-Windows-x64-setup.exe` | Run it |
| Windows (x64) | `octant_<version>_x64_en-US.msi` | Run it, or `msiexec /i` it for managed installs |
| Linux (x86_64) | `Octant-Linux-x86_64.AppImage` | `chmod +x Octant-Linux-x86_64.AppImage`, then run it |
| Debian, Ubuntu | `octant_<version>_amd64.deb` | `sudo apt install ./octant_*.deb` |
| Arch | `octant_<version>_x86_64.tar.gz` and `PKGBUILD` | `makepkg -si` in the folder holding both |

The buttons at the top download the `.dmg`, the `-setup.exe` and the `.AppImage`, named without their version so the links always reach the latest release.

The installers are not signed yet, so the first launch asks for confirmation:

- **macOS** says Octant "cannot be opened because Apple cannot verify it". Right-click Octant in Applications and choose **Open**, or click **Open Anyway** in System Settings > Privacy & Security.
- **Windows** SmartScreen says "Windows protected your PC": click **More info**, then **Run anyway**.

</details>

### From source

With [Rust installed](https://rust-lang.org/tools/install/) and [CMake](https://cmake.org/download/), which builds the bundled NetCDF and HDF5 libraries (macOS: `brew install cmake`, Debian/Ubuntu: `sudo apt install cmake`, Windows: included with the Visual Studio Build Tools' C++ workload):

```bash
cargo install octant
octant                      # or: octant <url or path>
```

If the build fails because a dependency released a breaking update, install with the exact versions Octant was tested with:

```bash
cargo install octant --locked
```

## Data

- [Zarr](https://zarr.dev/) v2 and v3 (including sharded arrays) and [Icechunk](https://icechunk.io/) repositories, from local disk, S3, Google Cloud Storage, Azure, or HTTP. Only the chunks needed for the current view are read.
- [OME-Zarr](https://ngff.openmicroscopy.org/) images, label images, and plates, with their OMERO channel settings.
- GeoTIFF and Cloud-Optimized GeoTIFF (COG).
- [NetCDF](https://www.unidata.ucar.edu/software/netcdf/) and [HDF5](https://www.hdfgroup.org/solutions/hdf5/) (desktop).
- Multiscale pyramids (OME-Zarr, GeoZarr `multiscales`, GeoTIFF overviews): Octant reads the level that fits the screen and loads finer detail as you zoom.
- A built-in catalog of public datasets, from climate reanalysis to microscopy.

## Viewing

- Heatmaps, line plots, surfaces, globes, volumes (direct volume rendering and intensity projections), and point clouds.
- Up to four overlays on a heatmap, each with its own colormap and opacity.
- True- and false-color, CMYK, and multichannel composites, plus label images.
- Animation along any dimension, with buffered playback.
- Over 200 colormaps from their original sources, custom colormaps, and opacity curves.
- Interactive hover readouts, coastlines, and scale bars.
- Figure export to PNG, JPEG, WebP, SVG, and PDF.
- An interface in 15 languages.

Rendering is powered by [`wgpu`](https://wgpu.rs/) (Vulkan, Metal, DirectX, WebGPU, and WebGL2) with an [`egui`](https://github.com/emilk/egui) interface.

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

Bundled material retains its own licenses, listed with attributions in the files below and in the application under **About**:

- Colormaps: [assets/colormaps/LICENSES.md](assets/colormaps/LICENSES.md) (CC0, MIT, BSD-3-Clause, Apache-2.0, the Matplotlib license, CC BY 4.0 for Colorcet and CARTOColors, and a few credited public-domain palettes).
- Rust crates, egui's fonts, and (on desktop) the netCDF-C and HDF5 libraries: [assets/licenses/THIRD_PARTY_LICENSES.md](assets/licenses/THIRD_PARTY_LICENSES.md), generated by [`cargo-about`](https://github.com/EmbarkStudios/cargo-about).
- Chinese, Japanese, and Korean fonts (Noto Sans CJK, SIL Open Font License 1.1): [assets/fonts/LICENSE.md](assets/fonts/LICENSE.md).

We have made a good-faith effort to credit and comply with the licenses of all bundled materials. If you hold rights to material you believe is used without proper permission or attribution, please [open an issue](https://github.com/lazarusA/octant/issues): we will promptly correct the attribution, add the required notice, or remove the material.
