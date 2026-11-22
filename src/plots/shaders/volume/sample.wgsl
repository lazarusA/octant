// Volume sampling shared by the dense and bricked variants: missing
// values, logical coordinates, opacity per reference step and the ray lattice.

// Returned for missing voxels; every mode treats |v| > 1e30 as missing.
const MISSING: f32 = 3.0e38;

// Length the DVR opacity (and NaN and clip colors' alpha) is defined for:
// 64 samples across the box (`reference_step`), so Density keeps its
// meaning at any quality.
const REFERENCE_STEP: f32 = 1.0 / 64.0;

// A ray's samples: `count` steps of `step` (unit-box units) from `start`, each
// `step_refs` reference steps long (`reference_step`). `entry` is where the ray enters the volume
// and `entry_normal` the world normal of the face there (facing the viewer);
// `view` points toward the viewer and `light` toward the headlight (world).
struct Ray {
    start: vec3<f32>,
    step: vec3<f32>,
    count: i32,
    step_refs: f32,
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

// Unit-box position (x right, y up, z toward the viewer) to logical texture
// coordinates: rows run top-down and planes front to back, as stored.
fn logical_coord(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(p.x, 1.0 - p.y, 1.0 - p.z);
}

// World length opacities are defined for along world direction `dir`:
// REFERENCE_STEP of the box's depth through its center along `dir`, at most
// of 1 (the box's longest side). A cube is unchanged; a box thin along one
// axis seen through its thickness is as opaque as a cube through its depth
// (not the canvas showing through it), and seen along its long sides as
// before, so Density still opens it up.
fn reference_step(dir: vec3<f32>) -> f32 {
    let depth = world_scale() / max(abs(dir), vec3<f32>(1e-6));
    return REFERENCE_STEP * min(1.0, min(depth.x, min(depth.y, depth.z)));
}

// Opacity of one step `steps` reference steps long, for a sample whose
// opacity is defined per reference step: view- and quality-independent.
fn corrected_alpha(alpha: f32, steps: f32) -> f32 {
    return 1.0 - pow(1.0 - clamp(alpha, 0.0, 0.9999), steps);
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
