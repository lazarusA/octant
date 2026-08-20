---
name: octant-rendering-wgpu
description: >-
  Guidelines for WGPU rendering pipelines, WGSL shaders, uniform buffer alignments,
  and egui-wgpu paint callbacks in Octant. Use when modifying or creating renderers in src/plots/.
---

# Octant WGPU Rendering Skill

This skill covers working with `wgpu` pipelines, WGSL shaders, buffer management, and integration with `egui` inside `src/plots/`.

## Renderers Architecture

Octant contains six core WGPU renderers (located in `src/plots/`):
1. **`MatrixRenderer` (`src/plots/heatmap.rs`)**: 2D heatmaps and raster scalar fields with bilinear/nearest interpolation.
2. **`LineRenderer` (`src/plots/line.rs`)**: 1D time-series and profile line plots with dynamic capacity reallocation.
3. **`PointCloudRenderer` (`src/plots/point_cloud.rs`)**: 3D point cloud coordinate visualization.
4. **`SphereRenderer` (`src/plots/sphere.rs`)**: Geospatial global projection onto 3D spheres.
5. **`SurfaceRenderer` (`src/plots/surface.rs`)**: 3D heightfield surface meshes with elevation scaling.
6. **`VolumeRenderer` (`src/plots/volume.rs`)**: 3D raymarched volumetric data with transfer functions (DVR, MIP, Isosurface).

## Best Practices & Invariants

### 1. Uniform Buffer Layouts (std140 & std430)
- Align all fields to WGSL alignment rules (`vec4` is 16-byte aligned, `mat4` is 64 bytes).
- Derive `#[repr(C)]`, `bytemuck::Pod`, and `bytemuck::Zeroable` on uniform structs.
- Add explicit padding fields (`_pad0`, `_pad1`) where necessary to guarantee alignment across all GPU hardware.

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlotColorParams {
    pub colormap: u32,
    pub cmin: f32,
    pub cmax: f32,
    pub use_nan_color: u32,
    pub use_lowclip: u32,
    pub use_highclip: u32,
    pub scale_type: u32,
    pub scale_param: f32,
    pub is_categorical: u32,
    pub num_categories: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub nan_color: [f32; 4],
    pub lowclip_color: [f32; 4],
    pub highclip_color: [f32; 4],
}
```

### 2. WGSL Shaders (`src/plots/shaders/*.wgsl`)
- Assemble shaders using `crate::assemble_plot_shader!(include_str!("shaders/..."))`.
- Colormaps are modularized in `src/plots/shaders/colormaps/` (Viridis, Plasma, Inferno, Magma, Turbo, Coolwarm, Cividis).
- Keep shaders compatible with WebGPU and WebGL2 (via `wgpu` downlevel flags).

### 3. Lock Safety & Poison Resilience in Renderers
- When accessing interior GPU buffer handles (`RwLock<LineGpuResources>`, etc.), **never use bare `unwrap()`**.
- Handle poison states with `if let Ok(...) = lock.read()` or `.unwrap_or_else(|p| p.into_inner())` to avoid UI crashes.

### 4. `egui_wgpu` Paint Callback
- In `src/plots/` renderers, paint passes execute inside `egui_wgpu::CallbackTrait`.
- Always use `setup_viewport_and_scissor` from `src/plots/common.rs` to clamp scissor rects strictly within physical surface bounds.
