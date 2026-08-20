---
name: octant-rendering-wgpu
description: >-
  Guidelines for WGPU rendering pipelines, WGSL shaders, uniform buffer alignments,
  and egui-wgpu paint callbacks in Octant. Use when modifying or creating renderers in src/plots/.
---

# Octant WGPU Rendering Skill

This skill covers working with `wgpu` pipelines, WGSL shaders, buffer management, and integration with `egui` inside `src/plots/`.

## Renderers Architecture

Octant contains three core WGPU renderers:
1. **`MatrixRenderer` (`src/plots/matrix.rs`)**: 2D heatmaps and raster scalar fields.
2. **`VolumeRenderer` (`src/plots/volume.rs`)**: 3D raymarched volumetric data with transfer functions.
3. **`SphereRenderer` (`src/plots/sphere.rs`)**: Geospatial global projection onto 3D spheres.

## Best Practices & Invariants

### 1. Uniform Buffer Layouts (std140 & std430)
- Align all fields to WGSL alignment rules (vec4 is 16-byte aligned, mat4 is 64 bytes).
- Derive `#[repr(C)]`, `bytemuck::Pod`, and `bytemuck::Zeroable` on uniform structs.
- Add explicit padding fields where necessary to guarantee alignment across all GPU hardware.

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4], // 64 bytes
    pub colormap_min: f32,        // 4 bytes
    pub colormap_max: f32,        // 4 bytes
    pub _padding: [f32; 2],       // 8 bytes (align to 16)
}
```

### 2. WGSL Shaders (`assets/shaders/*.wgsl`)
- Keep shaders compatible with WebGPU and WebGL2 (via `wgpu` downlevel flags).
- Verify sampler configurations: filtering modes (Linear vs Nearest) and clamp-to-edge addressing.

### 3. `egui_wgpu` Paint Callback
- In `src/plots/mod.rs` and renderers, render passes execute inside `egui::PaintCallback`.
- Ensure render passes cleanly restore viewport and pipeline state without corrupting `egui`'s UI pass.
