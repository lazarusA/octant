---
name: octant-rendering-wgpu
description: >-
  Guidelines for WGPU rendering pipelines, WGSL shaders, uniform buffer alignments,
  and egui-wgpu paint callbacks in Octant. Use when modifying or creating renderers in src/plots/.
---

# Octant WGPU Rendering Skill

This skill covers working with `wgpu` pipelines, WGSL shaders, buffer management, and integration with `egui` inside `src/plots/`.

## Renderers Architecture

Octant organizes WGPU renderers in `src/plots/`:
1. **`MatrixRenderer` (`src/plots/heatmap.rs`)**: 2D heatmaps and raster scalar fields with bilinear/nearest interpolation.
2. **`LineRenderer` (`src/plots/line.rs`)**: 1D time-series and profile line plots with dynamic capacity reallocation.
3. **`Mesh3DRenderer` (`src/plots/mesh.rs`)**: Unified GPU instanced 3D surface, globe, terrain, and voxel cube mesh renderer.
   - **`SphereRenderer` (`src/plots/sphere.rs`)**: Thin alias of `Mesh3DRenderer` compiled with `shaders/sphere.wgsl` and backface culling.
   - **`SurfaceRenderer` (`src/plots/surface.rs`)**: Thin alias of `Mesh3DRenderer` compiled with `shaders/surface.wgsl`.
4. **`PointCloudRenderer` (`src/plots/point_cloud.rs`)**: 3D point cloud billboard particles with coordinate/shift offsets.
5. **`VolumeRenderer` (`src/plots/volume.rs`)**: 3D raymarched volumetric data with transfer functions (DVR, MIP, Isosurface).

## Adding New 3D Geometries (Hexagons, Prisms, Custom Meshes)

To add a new 3D geometry shape (such as Hexagonal columns, icosahedra, or discrete global grid cells):

### Approach 1: Add as a Mode to `Mesh3DRenderer`
1. Define the unit template builder in `src/plots/common.rs`:
   ```rust
   pub fn build_unit_hex_prism<V, F>(mut make_vertex: F) -> (Vec<V>, Vec<u32>)
   where
       F: FnMut([f32; 3], [f32; 2], [f32; 3]) -> V,
   {
       // Return unit-scale vertices and indices
   }
   ```
2. Allocate template buffers once on device initialization in `src/plots/mesh.rs`:
   ```rust
   let (hex_verts, hex_indices) = Self::build_unit_hex_prism();
   let hex_vertex_buffer = create_buffer(device, &hex_verts);
   let hex_index_buffer = create_buffer(device, &hex_indices);
   ```
3. In `Mesh3DCallback::paint()`, branch on `self.params.mode` to bind the new vertex/index buffers and invoke `rpass.draw_indexed(0..indices_len, 0, 0..self.renderer.num_instances)`.
4. Handle the shape placement and elevation scaling in the corresponding WGSL vertex shader (`vs_main`).

### Approach 2: Add as a Dedicated Plot Type
1. Define a constructor in a new file `src/plots/<geom>.rs`:
   ```rust
   pub type HexRenderer = Mesh3DRenderer;
   pub type HexCallback = Mesh3DCallback;

   impl HexRenderer {
       pub fn new_hex(device: &wgpu::Device, target_format: wgpu::TextureFormat, matrix_data: &[f32], width: usize, height: usize) -> Self {
           let shader_source = crate::assemble_plot_shader!(include_str!("shaders/hex.wgsl"));
           Self::new(device, target_format, shader_source, Some(wgpu::Face::Back), matrix_data, width, height)
       }
   }
   ```
2. Register `PlotType::<NewGeom>` in `src/plots/mod.rs`.
3. Add a dispatch match arm in `OctantApp::paint_active_plot()` in `src/app/pipeline.rs`.

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

### 4. GPU Instancing & Vertex Pulling (Standard for All Grid Formats)
- **Zero-Allocation GPU Architecture**: Never allocate multi-megabyte/gigabyte vertex buffers on the CPU for grid data. All structured and discrete global grids must use lightweight GPU templates or vertex pulling:
  - **Regular Structured Grids (2D/3D)**: Unit Quad (4 vertices, `build_unit_quad_mesh`) or Unit Cube (24 vertices, `build_unit_cube_mesh`).
  - **Discrete Global Grids (HEALPix, Cubed-Sphere)**: Unit Quad (4 vertices) with analytical coordinate synthesis in WGSL.
  - **Icosahedral Grids (ICON)**: Unit Triangle (3 vertices) with icosahedral barycentric synthesis, or GPU vertex pulling from node coordinate storage buffers.
  - **Unstructured Topology (UGRID, MPAS)**: GPU Vertex Pulling—store static `node_coords` and `triangle_indices` in GPU storage buffers; pull vertices directly in the shader without recreating CPU meshes on dataset or variable changes.
  - **Lines / 1D Profiles**: Index-driven draw calls (`rpass.draw(0..profile_length, 0..line_count)`).

### 5. Valid Use Cases for CPU Mesh Generation
CPU mesh generation is reserved strictly for non-grid geometric processing and export workflows:
- **GIS Vector Polygons**: Arbitrary polygon boundaries, coastlines, and GeoJSON country borders requiring polygon clipping and triangulation (e.g., Earcut/CDT).
- **Streamlines & Particle Traces**: Dynamic numerical integration of particles through velocity vector fields (CFD/wind) to generate 3D ribbon or tube geometry.
- **Explicit 3D Model Export**: Marching Cubes or Dual Contouring when exporting polygonal files (.stl, .obj, .gltf) to disk.
- **UI & Annotations**: Viewport orientation compasses, 3D coordinate triads, and text label overlays.

### 6. Analytical Coordinate & Normal Synthesis in WGSL
- Pass the raw tensor data as a storage buffer (`@group(0) @binding(1) var<storage, read> data_buffer: array<f32>;`).
- Decode grid coordinates inside `vs_main` using `@builtin(instance_index)`:
  - **2D Regular Grid**: `let cell_x = instance_idx % uniforms.width; let cell_y = instance_idx / uniforms.width;`
  - **3D Regular Grid**: `let cell_x = instance_idx % width; let cell_y = (instance_idx / width) % height; let cell_z = instance_idx / (width * height);`
  - **HEALPix**: Decode pixel index to base diamond face $(0..11)$ and internal coordinates $(x, y) \to (\theta, \phi)$.
- Compute geometry analytically in the vertex shader:
  - **Spherical projections**: Compute $(u, v)$ and Cartesian coordinates (`x = radius * cos(lat) * sin(lon)`, etc.) in parallel on GPU cores.
  - **Lighting normals**: Compute finite-difference gradients from neighboring storage buffer cells (`dh_du = dr(val_right) - dr(val_left)`).
- **Vertex Shader Fast Culling**: If a cell/point is `NaN` or clipped by color range, immediately return `out.position = vec4<f32>(0.0, 0.0, 0.0, 0.0)` to skip fragment rasterization completely.

### 7. Safe Buffer Updates
- Always use `super::common::safe_write_buffer(queue, buffer, data, label)` when writing data slices. It guards against destination buffer overruns when switching datasets or projections.

### 8. `egui_wgpu` Paint Callback
- In `src/plots/` renderers, paint passes execute inside `egui_wgpu::CallbackTrait`.
- Always use `setup_viewport_and_scissor` from `src/plots/common.rs` to clamp scissor rects strictly within physical surface bounds.
