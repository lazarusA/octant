// Volume sampling: unit-box positions to texture coordinates, trilinear and
// exact reads, and the ray lattice.

// Returned for missing voxels; every mode treats |v| > 1e30 as missing.
const MISSING: f32 = 3.0e38;

// World length the DVR opacity (and NaN and clip colors' alpha) is
// defined for: 64 samples across a unit box, so Density keeps its meaning at
// any quality.
const REFERENCE_STEP: f32 = 1.0 / 64.0;

// A ray's samples: `count` steps of `step` (unit-box units) from `start`, each
// `step_world` long in world units. `entry` is where the ray enters the volume
// and `entry_normal` the world normal of the face there (facing the viewer);
// `view` points toward the viewer and `light` toward the headlight (world).
struct Ray {
    start: vec3<f32>,
    step: vec3<f32>,
    count: i32,
    step_world: f32,
    entry: vec3<f32>,
    entry_normal: vec3<f32>,
    view: vec3<f32>,
    light: vec3<f32>,
};

fn is_missing(v: f32) -> bool {
    return abs(v) > 1e30;
}

fn is_composite() -> bool {
    return uniforms.composite != 0u;
}

fn volume_dims() -> vec3<f32> {
    return vec3<f32>(textureDimensions(volume_values));
}

fn volume_shift() -> vec3<u32> {
    return vec3<u32>(uniforms.shift_x, uniforms.shift_y, uniforms.shift_z);
}

// Unit-box position (x right, y up, z toward the viewer) to logical texture
// coordinates: rows run top-down and planes front to back, as stored.
fn logical_coord(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(p.x, 1.0 - p.y, 1.0 - p.z);
}

// Sampler coordinate of `p`: clamped to the first and last texel centers
// (ClampToEdge at the box faces), then rotated by the ring-buffer shift, so the
// Repeat sampler blends across the seam inside the volume only.
fn texture_coord(p: vec3<f32>) -> vec3<f32> {
    let dims = volume_dims();
    let half_texel = 0.5 / dims;
    return clamp(logical_coord(p), half_texel, 1.0 - half_texel) + vec3<f32>(volume_shift()) / dims;
}

fn is_valid_at(uvw: vec3<f32>) -> bool {
    if (uniforms.has_invalid == 0u) {
        return true;
    }
    return textureSampleLevel(volume_validity, volume_sampler, uvw, 0.0).r >= 0.5;
}

// Trilinear scalar at `p`, or MISSING.
fn sample_scalar(p: vec3<f32>) -> f32 {
    let uvw = texture_coord(p);
    if (!is_valid_at(uvw)) {
        return MISSING;
    }
    return fetch_trilinear(uvw).r;
}

// Trilinear RGB composite color (alpha = brightest channel), or zero where missing.
fn sample_composite(p: vec3<f32>) -> vec4<f32> {
    let uvw = texture_coord(p);
    if (!is_valid_at(uvw)) {
        return vec4<f32>(0.0);
    }
    return fetch_trilinear(uvw);
}

fn wrap_texel(i: vec3<i32>, n: vec3<i32>) -> vec3<i32> {
    return ((i % n) + n) % n;
}

// Texel holding `p` (nearest voxel), ring-buffer shift applied.
fn exact_texel(p: vec3<f32>) -> vec3<u32> {
    let dims = vec3<u32>(textureDimensions(volume_values));
    let l = clamp(logical_coord(p), vec3<f32>(0.0), vec3<f32>(1.0));
    return (min(vec3<u32>(l * vec3<f32>(dims)), dims - 1u) + volume_shift()) % dims;
}

fn texel_missing(t: vec3<u32>) -> bool {
    return uniforms.has_invalid != 0u && textureLoad(volume_validity, t, 0).r < 0.5;
}

// Exact value of the voxel containing `p`, or MISSING.
fn sample_exact(p: vec3<f32>) -> f32 {
    let t = exact_texel(p);
    if (texel_missing(t)) {
        return MISSING;
    }
    return textureLoad(volume_values, t, 0).r;
}

// Opacity of one step of `step_world` for a sample whose opacity is defined
// per REFERENCE_STEP: view- and quality-independent accumulation.
fn corrected_alpha(alpha: f32, step_world: f32) -> f32 {
    return 1.0 - pow(1.0 - clamp(alpha, 0.0, 0.9999), step_world / REFERENCE_STEP);
}

fn premultiply(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(c.rgb * c.a, c.a);
}

fn sample_pos(ray: Ray, i: i32) -> vec3<f32> {
    return ray.start + f32(i) * ray.step;
}

// Sample before `i` (the ray entry for the first): where a surface search starts.
fn previous_pos(ray: Ray, i: i32) -> vec3<f32> {
    return select(sample_pos(ray, i - 1), ray.entry, i == 0);
}
