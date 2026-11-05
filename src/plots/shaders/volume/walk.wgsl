// Exact voxel walk (3D DDA, Amanatides-Woo): every voxel a ray's segment
// crosses, in order, each once, so nothing is skipped however large the
// volume. Walks logical texel space (texel k spans [k, k + 1)); the stored
// texel (ring shift applied) follows step by step, without a modulo.

// Parameter of an axis the segment runs parallel to: never reached.
const WALK_NEVER: f32 = 1e30;

struct Walk {
    // Logical voxel and its stored texel.
    cell: vec3<i32>,
    texel: vec3<i32>,
    dims: vec3<i32>,
    // +1 or -1 along each axis, 0 where the segment runs parallel to it.
    dir: vec3<i32>,
    // Segment parameter (0 at the entry, 1 at the exit) of the next face on
    // each axis, and of one voxel's width along it.
    t_max: vec3<f32>,
    t_delta: vec3<f32>,
    // Parameter where the walk entered the current voxel.
    t: f32,
    // Axis of the face it entered through, -1 for the segment's first voxel.
    axis: i32,
};

// Logical texel position of unit-box position `p`.
fn walk_texel_pos(p: vec3<f32>) -> vec3<f32> {
    return logical_coord(p) * volume_dims();
}

// The walk along the ray's whole segment, from its entry to its exit.
fn walk_begin(ray: Ray) -> Walk {
    let a = walk_texel_pos(ray.entry);
    let span = walk_texel_pos(ray.entry + ray.step * f32(ray.count)) - a;
    return walk_from(a, span, 0.0);
}

// The walk over the segment `a + t * span` from parameter `t0` on.
fn walk_from(a: vec3<f32>, span: vec3<f32>, t0: f32) -> Walk {
    var w: Walk;
    w.dims = vec3<i32>(textureDimensions(volume_values));
    // Nudged along the ray, so a start on a face picks the voxel ahead.
    let start = a + span * (t0 + 1e-6);
    w.cell = clamp(vec3<i32>(floor(start)), vec3<i32>(0), w.dims - 1);
    w.texel = (w.cell + vec3<i32>(volume_shift())) % w.dims;
    let parallel = span == vec3<f32>(0.0);
    let ahead = span > vec3<f32>(0.0);
    w.dir = select(select(vec3<i32>(-1), vec3<i32>(1), ahead), vec3<i32>(0), parallel);
    let safe = select(span, vec3<f32>(1.0), parallel);
    let face = vec3<f32>(w.cell) + select(vec3<f32>(0.0), vec3<f32>(1.0), ahead);
    w.t_max = select((face - a) / safe, vec3<f32>(WALK_NEVER), parallel);
    w.t_delta = select(abs(1.0 / safe), vec3<f32>(WALK_NEVER), parallel);
    w.t = t0;
    w.axis = -1;
    return w;
}

// Most voxels any walk visits: one per face crossed on each axis.
fn walk_limit(w: Walk) -> i32 {
    return w.dims.x + w.dims.y + w.dims.z + 3;
}

// Parameter where the walk leaves the current voxel.
fn walk_exit(w: Walk) -> f32 {
    return min(min(w.t_max.x, w.t_max.y), w.t_max.z);
}

// Moves to the next voxel; returns false once the segment ends.
fn walk_step(w: ptr<function, Walk>) -> bool {
    let m = (*w).t_max;
    var axis = 0;
    if (m.y < m.x) {
        axis = 1;
    }
    if (m.z < m[axis]) {
        axis = 2;
    }
    let t = m[axis];
    if (t >= 1.0) {
        return false;
    }
    (*w).t = t;
    (*w).t_max[axis] = t + (*w).t_delta[axis];
    let d = (*w).dir[axis];
    let cell = (*w).cell[axis] + d;
    let n = (*w).dims[axis];
    if (cell < 0 || cell >= n) {
        return false;
    }
    (*w).cell[axis] = cell;
    var tx = (*w).texel[axis] + d;
    tx = select(tx, 0, tx >= n);
    tx = select(tx, n - 1, tx < 0);
    (*w).texel[axis] = tx;
    (*w).axis = axis;
    return true;
}

// Whether the current voxel is missing.
fn walk_missing(w: Walk) -> bool {
    return uniforms.has_invalid != 0u && textureLoad(volume_validity, w.texel, 0).r < 0.5;
}

// The current voxel's texel: scalar in `r`, composite color in `rgba`.
fn walk_texel(w: Walk) -> vec4<f32> {
    return textureLoad(volume_values, w.texel, 0);
}

// Unit-box normal of the face the walk entered the current voxel through
// (facing back along the ray), or `entry_normal` for the first voxel.
fn walk_face_normal(w: Walk, entry_normal: vec3<f32>) -> vec3<f32> {
    if (w.axis < 0) {
        return entry_normal;
    }
    var n = vec3<f32>(0.0);
    n[w.axis] = -f32(w.dir[w.axis]);
    // Logical rows run top-down and planes front to back (`logical_coord`).
    return n * vec3<f32>(1.0, -1.0, -1.0);
}
