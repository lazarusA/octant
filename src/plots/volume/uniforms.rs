//! Uniform block of the volume raymarcher and the per-frame parameters it is
//! built from (zero-allocation `Copy` structs).

use crate::plots::common::PlotColorParams;
use bytemuck::{Pod, Zeroable};

/// Mirrors WGSL `Uniforms` in `shaders/volume/bindings.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct VolumeUniforms {
    pub clip_planes: [[f32; 4]; 8],
    pub light_color: [f32; 3],
    pub num_clip_planes: u32,
    pub ambient: [f32; 3],
    pub shininess: f32,
    pub light_direction: [f32; 3],
    pub algorithm: u32,
    pub isovalue: f32,
    pub isorange: f32,
    pub absorption: f32,
    /// Samples per voxel crossed by each ray.
    pub quality: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub attenuation: f32,
    /// Non-zero while any voxel is missing; the shader reads validity only then.
    pub has_invalid: u32,
    /// Non-zero when the value texture holds RGB composite colors.
    pub composite: u32,
    pub rotation_y: f32,
    pub rotation_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub transparency: u32,
    pub _pad1: u32,
    pub color: PlotColorParams,
}

#[derive(Copy, Clone, Debug)]
pub struct VolumeUniformParams {
    pub color: PlotColorParams,
    pub rot_y: f32,
    pub rot_x: f32,
    pub aspect_x: f32,
    pub aspect_y: f32,
    pub aspect_z: f32,
    pub zoom: f32,
    pub opacity_scale: f32,
    /// Samples per voxel crossed by each ray.
    pub quality: f32,
    pub algorithm: u32,
    pub isovalue: f32,
    pub isorange: f32,
    pub attenuation: f32,
    pub screen_aspect: f32,
    pub shift_x: u32,
    pub shift_y: u32,
    pub shift_z: u32,
    pub transparency: bool,
}

/// Texture state the uniforms report alongside the frame parameters.
#[derive(Copy, Clone, Debug)]
pub struct VolumeState {
    pub dims: [u32; 3],
    pub has_invalid: bool,
    pub composite: bool,
}

impl VolumeUniformParams {
    pub fn to_uniforms(&self, state: VolumeState) -> VolumeUniforms {
        let [width, height, depth] = state.dims;
        VolumeUniforms {
            clip_planes: [[0.0; 4]; 8],
            light_color: [1.0, 1.0, 1.0],
            num_clip_planes: 0,
            ambient: [0.2, 0.2, 0.2],
            shininess: 32.0,
            light_direction: [1.0, 1.0, 1.0],
            algorithm: self.algorithm,
            isovalue: self.isovalue,
            isorange: self.isorange,
            absorption: self.opacity_scale,
            quality: self.quality,
            diffuse: 0.8,
            specular: 0.2,
            attenuation: self.attenuation,
            has_invalid: u32::from(state.has_invalid),
            composite: u32::from(state.composite),
            rotation_y: self.rot_y,
            rotation_x: self.rot_x,
            aspect_x: self.aspect_x,
            aspect_y: self.aspect_y,
            aspect_z: self.aspect_z,
            zoom: self.zoom,
            width,
            height,
            depth,
            screen_aspect: self.screen_aspect,
            shift_x: self.shift_x,
            shift_y: self.shift_y,
            shift_z: self.shift_z,
            transparency: u32::from(self.transparency),
            _pad1: 0,
            color: self.color,
        }
    }
}
