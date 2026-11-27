<h1 align="center"><picture><source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/ndvis/octant/17489ba8410364d3fd0d11c697eb809f990e3486/assets/brand/svg/octant-logo-dark.svg"><img alt="" src="https://raw.githubusercontent.com/ndvis/octant/17489ba8410364d3fd0d11c697eb809f990e3486/assets/brand/svg/octant-logo-light.svg" height="40" align="center"></picture> octant</h1>

<p align="center">
  <a href="https://github.com/ndvis/octant"><img alt="github" src="https://img.shields.io/badge/github-ndvis/octant-8da0cb?logo=github"></a>
  <a href="https://ndvis.github.io/octant/"><img alt="Live Demo" src="https://img.shields.io/badge/demo-online%20(wasm)-blue?logo=webassembly&logoColor=white"></a>
  <a href="https://crates.io/crates/octant"><img alt="Latest version" src="https://img.shields.io/crates/v/octant.svg"></a>
  <a href="https://github.com/ndvis/octant/actions/workflows/rust.yml"><img alt="Build Status" src="https://github.com/ndvis/octant/actions/workflows/rust.yml/badge.svg"></a>
  <a href="https://github.com/ndvis/octant/blob/main/LICENSE-MIT"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <a href="https://github.com/ndvis/octant/blob/main/LICENSE-APACHE"><img alt="Apache" src="https://img.shields.io/badge/license-Apache-blue.svg"></a>
</p>

<p align="center">
  <b>Download the desktop app</b> · or <a href="https://ndvis.github.io/octant/">try it in your browser.</a>
</p>

<p align="center">
  <a href="https://github.com/ndvis/octant/releases/latest/download/Octant-macOS-arm64.dmg"><img alt="Download for macOS (Apple silicon)" src="https://img.shields.io/badge/macOS-Apple%20silicon-3a3f44?style=for-the-badge&labelColor=121416"></a>
  <a href="https://github.com/ndvis/octant/releases/latest/download/Octant-Windows-x64-setup.exe"><img alt="Download for Windows (x64)" src="https://img.shields.io/badge/Windows-x64-3a3f44?style=for-the-badge&labelColor=121416"></a>
  <a href="https://github.com/ndvis/octant/releases/latest/download/Octant-Linux-x86_64.AppImage"><img alt="Download for Linux (x86_64 AppImage)" src="https://img.shields.io/badge/Linux-AppImage-3a3f44?style=for-the-badge&labelColor=121416"></a>
</p>

An interactive viewer for N-dimensional datasets, stored locally or in cloud object storage. Built in Rust with GPU rendering, available on desktop and in the browser.

https://github.com/user-attachments/assets/c749e1d1-9159-4928-9e05-9490fb466607

![Octant screenshot](https://raw.githubusercontent.com/ndvis/octant/refs/heads/main/assets/octant_ws10.png)

## Install

<details>
<summary>Every installer, and how to install it</summary>

Each [release](https://github.com/ndvis/octant/releases/latest) holds:

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

To save a share link's plot as an image without clicking through the app (for thumbnails or scripts), run `octant --snapshot out.png "<share link>"`. Add `--size 800x450` (the image's exact pixels, the plot scaled and centre-cropped to fit), `--timeout 120`, `--no-colorbar` or `--no-axes` as needed; the format follows the extension (png, jpg, webp, svg, pdf). Octant exits with 0 once the image is saved, 1 on a failure (the reason is printed), and 2 for a wrong command line. The window shows on screen while it captures.

If the build fails because a dependency released a breaking update, install with the exact versions Octant was tested with:

```bash
cargo install octant --locked
```

## Data

- [Zarr](https://zarr.dev/) v2 and v3 (including sharded arrays) and [Icechunk](https://icechunk.io/) repositories, from local disk, S3, Google Cloud Storage, Azure, or HTTP. Only the chunks needed for the current view are read.
- [OME-Zarr](https://ngff.openmicroscopy.org/) images, label images, and plates, with their OMERO channel settings.
- GeoTIFF and Cloud-Optimized GeoTIFF (COG).
- [NetCDF](https://www.unidata.ucar.edu/software/netcdf/) (classic and NetCDF-4), [HDF5](https://www.hdfgroup.org/solutions/hdf5/), and MATLAB v7.3 `.mat` files, local or remote (HTTP or S3), on desktop and in the browser.
- GRIB2 files, local or remote (HTTP or S3), on desktop and in the browser (JPEG2000-packed fields on desktop only). On desktop, NOAA and ECMWF sidecar indexes (`.idx`, `.index`) speed up remote files. Zarr and Icechunk stores built on GRIB files, such as [dynamical.org](https://dynamical.org/)'s, read too.
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

## Related tools

Many good viewers exist for scientific arrays, and most specialize in one domain, one format family, or one platform. **Octant** aims to be one viewer for N-dimensional data from climate to microscopy, read straight from cloud storage.

**Octant** is built first for the desktop, where it has your GPU, memory, and files to itself. The browser version runs the same code and is there for a quick look or a shared link.

What Octant brings together:

- **Earth-science and bioimaging data in one app**: from GRIB forecasts and NetCDF model output to OME-Zarr plates ([formats](#data)).
- **A native desktop app that streams from the cloud**: S3, Google Cloud Storage, Azure, or HTTP, with no download, server, Java, Julia, or Python.
- **Heatmaps, globes, and GPU volume rendering in the same window**, switched with a click on the same data.
- **One codebase for desktop and browser**: the web version is the same Rust compiled to WebAssembly, so a shared link opens the same view in a browser, or in the desktop app when pasted into its input field (for publicly reachable data).

The tools below each do their job well, and you may prefer one of them for its own strengths.

**Web**

| Tool | Runs on | Focus |
|---|---|---|
| [Browzarr](https://github.com/EarthyScience/Browzarr) | Browser | Zarr and NetCDF exploration in the browser, no backend |
| [gridlook](https://github.com/d70-t/gridlook) | Browser | Zarr, Icechunk, and NetCDF model output on native grids, on a 3D globe |
| [LexCube](https://www.lexcube.org/) | Browser, Jupyter | Interactive 3D space-time data cubes |
| [xcube Viewer](https://github.com/xcube-dev/xcube-viewer) | Browser, with an xcube server | Data cubes on a map |
| [Neuroglancer](https://github.com/google/neuroglancer) | Browser | Multiscale volumes, meshes, and skeletons (precomputed, N5, Zarr, OME-Zarr) |
| [Vizarr](https://github.com/hms-dbmi/vizarr) | Browser, Jupyter | 2D multichannel views of OME-Zarr and OME-TIFF |
| [Avivator](https://github.com/hms-dbmi/viv) | Browser | OME-TIFF and OME-Zarr viewer built on Viv |
| [Find Nuclei](https://find-nuclei.github.io) | Browser | Many microscopy formats in 2D and 3D; closed source, free to use |

**Desktop**

| Tool | Runs on | Focus |
|---|---|---|
| [Panoply](https://www.giss.nasa.gov/tools/panoply/) | Java | NetCDF, HDF, and GRIB maps and plots; remote files and THREDDS/OPeNDAP catalogs |
| [ncview](https://cirrus.ucsd.edu/ncview/) | X11 | Quick NetCDF slice browsing and movies |
| [VAPOR](https://www.vapor.ucar.edu/) | Native app | 3D visualization of atmosphere and ocean model output |
| [IDV](https://www.unidata.ucar.edu/software/idv/) | Java | 2D and 3D displays of models, satellite, radar, and observations |
| [QGIS](https://qgis.org/) | Native app | Geographic information system for raster and vector maps |
| [napari](https://github.com/napari/napari) | Python | N-dimensional images in 2D and 3D, with a plugin ecosystem |
| [BigDataViewer](https://imagej.net/plugins/bdv/) | Fiji plugin | Re-slicing very large multi-view volumes |
| [BigWarp](https://imagej.net/plugins/bigwarp) | Fiji plugin | Landmark-based image registration, streaming N5, Zarr, and HDF5 |
| [ParaView](https://www.paraview.org/) | Native app, also HPC and browser | General-purpose 3D scientific visualization |

Libraries for building your own viewer: [Kiln](https://github.com/kilnrender/kiln-render) (WebGPU volume rendering of OME-Zarr), [Viv](https://github.com/hms-dbmi/viv) (OME-Zarr and OME-TIFF layers for deck.gl), [zarr-layer](https://github.com/carbonplan/zarr-layer) (Zarr on MapLibre and Mapbox maps), and [deck.gl-raster](https://github.com/developmentseed/deck.gl-raster) (COG and Zarr layers for deck.gl).

Know a tool we missed? [Open an issue or a pull request](https://github.com/ndvis/octant/issues).

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

Bundled material retains its own licenses, listed with attributions in the files below and in the application under **About**:

- Colormaps: [assets/colormaps/LICENSES.md](assets/colormaps/LICENSES.md) (CC0, MIT, BSD-3-Clause, Apache-2.0, the Matplotlib license, CC BY 4.0 for Colorcet and CARTOColors, and a few credited public-domain palettes).
- Rust crates, egui's fonts, and (on desktop) the netCDF-C and HDF5 libraries: [assets/licenses/THIRD_PARTY_LICENSES.md](assets/licenses/THIRD_PARTY_LICENSES.md), generated by [`cargo-about`](https://github.com/EmbarkStudios/cargo-about).
- Chinese, Japanese, and Korean fonts (Noto Sans CJK, SIL Open Font License 1.1): [assets/fonts/LICENSE.md](assets/fonts/LICENSE.md).

We have made a good-faith effort to credit and comply with the licenses of all bundled materials. If you hold rights to material you believe is used without proper permission or attribution, please [open an issue](https://github.com/ndvis/octant/issues): we will promptly correct the attribution, add the required notice, or remove the material.
