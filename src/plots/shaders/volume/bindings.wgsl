// =================================================================================================
// Volume Raymarching Shader: bindings, uniforms and the bounding-box vertex stage.
//
// Rendering Modes (uniforms.algorithm):
//   0: Volume Raymarching (DVR)    - Front-to-back alpha compositing with step-corrected opacity
//   1: Maximum Intensity (MIP)     - Maximum intensity projection with density-weighted attenuation
//   2: Minimum Intensity (MinIP)   - Minimum intensity projection along the ray
//   3: Average Projection (X-ray)  - Average column scalar intensity (radiographic transmission)
//   4: Categorical Label Surface   - Binary foreground mask isosurface for segmented data
//   5: Absorption RGBA             - Classical optical absorption model
//   6: Additive RGBA               - Additive volume emission model
//   7: Indexed Discrete RGBA       - Palette-indexed discrete material rendering
//
// DVR returns premultiplied color; modes 1-7 keep their classic straight-alpha
// output, premultiplied in `fs_main`.
// =================================================================================================

struct Uniforms {
    camera: Camera,
    clip_planes: array<vec4<f32>, 8>,
    light_color: vec3<f32>,
    num_clip_planes: u32,
    ambient: vec3<f32>,
    shininess: f32,
    light_direction: vec3<f32>,
    algorithm: u32,
    isovalue: f32,
    isorange: f32,
    absorption: f32,
    quality: f32,
    diffuse: f32,
    specular: f32,
    attenuation: f32,
    has_invalid: u32,
    composite: u32,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    width: u32,
    height: u32,
    depth: u32,
    shift_x: u32,
    shift_y: u32,
    shift_z: u32,
    transparency: u32,
    lighting: u32,
    color: ColorUniforms,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Voxel values: R32Float scalars, or Rgba8Unorm colors (alpha = brightest
// channel) in RGB composite mode. Missing voxels hold their neighbors' average.
@group(0) @binding(1)
var volume_values: texture_3d<f32>;

// 1 where the voxel holds data, 0 where it is missing. Filtered, its 0.5 level
// is a smooth missing-data boundary.
@group(0) @binding(2)
var volume_validity: texture_3d<f32>;

// Linear, Repeat on every axis (see `texture_coord`).
@group(0) @binding(3)
var volume_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_vert: vec3<f32>,
};

const CUBE_CORNERS = array<vec3<f32>, 8>(
    vec3<f32>(-1.0, -1.0, 1.0),
    vec3<f32>(1.0, -1.0, 1.0),
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(-1.0, 1.0, 1.0),
    vec3<f32>(-1.0, -1.0, -1.0),
    vec3<f32>(1.0, -1.0, -1.0),
    vec3<f32>(1.0, 1.0, -1.0),
    vec3<f32>(-1.0, 1.0, -1.0),
);

// Counter-clockwise seen from outside, so culling front faces keeps the far side.
const CUBE_INDICES = array<u32, 36>(
    0u, 1u, 2u, 0u, 2u, 3u, // Front
    5u, 4u, 7u, 5u, 7u, 6u, // Back
    4u, 0u, 3u, 4u, 3u, 7u, // Left
    1u, 5u, 6u, 1u, 6u, 2u, // Right
    3u, 2u, 6u, 3u, 6u, 7u, // Top
    4u, 5u, 1u, 4u, 1u, 0u, // Bottom
);

// Camera-space vector (x right, y up, z toward the viewer) in object space.
fn camera_to_object(v: vec3<f32>) -> vec3<f32> {
    return camera_to_object_dir(uniforms.camera, v);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var corners = CUBE_CORNERS;
    var indices = CUBE_INDICES;
    let position = corners[indices[min(vertex_index, 35u)]];

    // Centered object position [-0.5, 0.5] scaled by aspect ratio
    let pos_3d = position * 0.5 * vec3<f32>(uniforms.aspect_x, uniforms.aspect_y, uniforms.aspect_z);

    out.frag_vert = pos_3d;
    out.position = camera_project(uniforms.camera, camera_view(uniforms.camera, pos_3d));
    return out;
}
