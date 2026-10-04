// =================================================================================================
// Volume Raymarching Shader: bindings, uniforms and the bounding-box vertex stage.
//
// Rendering Modes (uniforms.algorithm):
//   0: Volume Raymarching (DVR)    - Front-to-back alpha compositing with step-corrected opacity
//   1: Maximum Intensity (MIP)     - Maximum intensity projection with density-weighted attenuation
//   2: Minimum Intensity (MinIP)   - Minimum intensity projection along the ray
//   3: Average Projection (X-ray)  - Average column scalar intensity (radiographic transmission)
//   4: Categorical Label Surface   - Binary foreground mask isosurface for segmented data
//   5: Absorption RGBA             - Beer-Lambert optical absorption model
//   6: Additive RGBA               - Additive volume emission model
//   7: Indexed Discrete RGBA       - Palette-indexed discrete material rendering
//
// Every mode returns premultiplied color.
// =================================================================================================

struct Uniforms {
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
    rotation_y: f32,
    rotation_x: f32,
    aspect_x: f32,
    aspect_y: f32,
    aspect_z: f32,
    zoom: f32,
    width: u32,
    height: u32,
    depth: u32,
    screen_aspect: f32,
    shift_x: u32,
    shift_y: u32,
    shift_z: u32,
    transparency: u32,
    _pad1: u32,
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

// Transfer function: colormap RGB and DVR opacity at 256 scale positions.
@group(0) @binding(3)
var transfer_lut: texture_2d<f32>;

// Linear, Repeat on every axis (see `texture_coord`).
@group(0) @binding(4)
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

fn camera_distance() -> f32 {
    return clamp(uniforms.zoom, 0.1, 10.0);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var corners = CUBE_CORNERS;
    var indices = CUBE_INDICES;
    let position = corners[indices[min(vertex_index, 35u)]];

    // Centered object position [-0.5, 0.5] scaled by aspect ratio
    let pos_3d = position * 0.5 * vec3<f32>(uniforms.aspect_x, uniforms.aspect_y, uniforms.aspect_z);

    // 3D Camera rotation around Y and X axes
    let cy = cos(uniforms.rotation_y);
    let sy = sin(uniforms.rotation_y);
    let cx = cos(uniforms.rotation_x);
    let sx = sin(uniforms.rotation_x);
    let pos_y_rot = vec3<f32>(cy * pos_3d.x + sy * pos_3d.z, pos_3d.y, -sy * pos_3d.x + cy * pos_3d.z);
    let pos_rot = vec3<f32>(pos_y_rot.x, cx * pos_y_rot.y - sx * pos_y_rot.z, sx * pos_y_rot.y + cx * pos_y_rot.z);

    out.frag_vert = pos_3d;

    let dist_positive = max(camera_distance() - pos_rot.z, 0.001);
    let fov_scale = 1.6;
    let screen_asp = max(uniforms.screen_aspect, 0.1);
    let proj_x = (pos_rot.x * fov_scale) / screen_asp;
    let proj_y = pos_rot.y * fov_scale;

    let z_near = 0.01;
    let z_far = 50.0;
    let proj_z = (z_far / (z_far - z_near)) * dist_positive - (z_far * z_near / (z_far - z_near));

    out.position = vec4<f32>(proj_x, proj_y, proj_z, dist_positive);
    return out;
}
