// Exact voxel walk (3D DDA, Amanatides-Woo): every voxel a ray's segment
// crosses, in order, each once, so nothing is skipped however large the
// volume. Walks logical texel space (texel k spans [k, k + 1)); the stored
// texel (ring shift applied) follows step by step, without a modulo.

// Parameter of an axis the segment runs parallel to: never reached.
const WALK_NEVER: f32 = 1e30;

struct Walk {
    // The segment, `a + t * span` in logical texel positions.
    a: vec3<f32>,
    span: vec3<f32>,
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
    var w: Walk;
    w.a = a;
    w.span = span;
    w.dims = vec3<i32>(textureDimensions(volume_values));
    let parallel = span == vec3<f32>(0.0);
    let ahead = span > vec3<f32>(0.0);
    w.dir = select(select(vec3<i32>(-1), vec3<i32>(1), ahead), vec3<i32>(0), parallel);
    // Nudged along the ray, so a start on a face picks the voxel ahead
    // (`voxel_walk.rs` `walk_voxels` nudges alike).
    let start = a + vec3<f32>(w.dir) * 1e-3;
    w.cell = clamp(vec3<i32>(floor(start)), vec3<i32>(0), w.dims - 1);
    w.texel = (w.cell + vec3<i32>(volume_shift())) % w.dims;
    let safe = select(span, vec3<f32>(1.0), parallel);
    let face = vec3<f32>(w.cell) + select(vec3<f32>(0.0), vec3<f32>(1.0), ahead);
    w.t_max = select((face - a) / safe, vec3<f32>(WALK_NEVER), parallel);
    w.t_delta = select(abs(1.0 / safe), vec3<f32>(WALK_NEVER), parallel);
    w.t = 0.0;
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

// Lanes of `axis` (0, 1 or 2): writes go through `select` on it, since
// Direct3D's FXC cannot assign a vector lane picked at run time.
fn axis_mask(axis: i32) -> vec3<bool> {
    return vec3<i32>(0, 1, 2) == vec3<i32>(axis);
}

// Moves to the next voxel; returns false once the segment ends.
fn walk_step(w: ptr<function, Walk>) -> bool {
    let m = (*w).t_max;
    let t = min(min(m.x, m.y), m.z);
    if (t >= 1.0) {
        return false;
    }
    // The first axis reaching `t`: x, then y, then z on ties.
    let axis = select(select(2, 1, m.y == t), 0, m.x == t);
    let on = axis_mask(axis);
    (*w).t = t;
    (*w).t_max = select(m, m + (*w).t_delta, on);
    let d = select(vec3<i32>(0), (*w).dir, on);
    let n = (*w).dims;
    let cell = (*w).cell + d;
    if (any(cell < vec3<i32>(0)) || any(cell >= n)) {
        return false;
    }
    (*w).cell = cell;
    var tx = (*w).texel + d;
    tx = select(tx, vec3<i32>(0), tx >= n);
    tx = select(tx, n - 1, tx < vec3<i32>(0));
    (*w).texel = tx;
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
    let n = select(vec3<f32>(0.0), -vec3<f32>(w.dir), axis_mask(w.axis));
    // Logical rows run top-down and planes front to back (`logical_coord`).
    return n * vec3<f32>(1.0, -1.0, -1.0);
}

// Segment parameter where `a + t * span` (logical texel positions) leaves
// the stored block holding logical voxel `cell` (stored texel `texel`), and
// the axis it leaves through. A block the ring buffer's seam splits is left
// at the volume's face.
fn block_exit(a: vec3<f32>, span: vec3<f32>, cell: vec3<i32>, texel: vec3<i32>, dims: vec3<i32>) -> vec2<f32> {
    let first = (texel >> vec3<u32>(BLOCK_SHIFT)) << vec3<u32>(BLOCK_SHIFT);
    let lo = cell - (texel - first);
    let hi = vec3<f32>(min(lo + min(vec3<i32>(BLOCK_EDGE), dims - first), dims));
    let ahead = span > vec3<f32>(0.0);
    let parallel = span == vec3<f32>(0.0);
    let face = select(vec3<f32>(max(lo, vec3<i32>(0))), hi, ahead);
    let t = select((face - a) / select(span, vec3<f32>(1.0), parallel), vec3<f32>(WALK_NEVER), parallel);
    var axis = 0;
    if (t.y < t.x) {
        axis = 1;
    }
    if (t.z < t[axis]) {
        axis = 2;
    }
    return vec2<f32>(t[axis], f32(axis));
}

// Whether the walk's block may hold what `kind` asks, read only when the
// walk enters another block (`last`, `open` carry the previous answer).
fn walk_block_open(
    w: Walk,
    last: ptr<function, vec3<i32>>,
    open: ptr<function, bool>,
    kind: u32,
    maximum: f32,
) -> bool {
    let block = w.texel >> vec3<u32>(BLOCK_SHIFT);
    if (any(block != *last)) {
        *last = block;
        *open = block_open(textureLoad(volume_blocks, block, 0), kind, maximum);
    }
    return *open;
}

// Moves the walk past its block, crossing on each axis the faces before the
// block's exit (at least one on the exit axis); returns false once the
// segment or the volume ends.
fn walk_skip_block(w: ptr<function, Walk>) -> bool {
    let e = block_exit((*w).a, (*w).span, (*w).cell, (*w).texel, (*w).dims);
    if (e.x >= 1.0) {
        return false;
    }
    let axis = i32(e.y);
    let still = (*w).dir == vec3<i32>(0);
    var n = vec3<i32>(max(floor((e.x - (*w).t_max) / (*w).t_delta) + 1.0, vec3<f32>(0.0)));
    n = select(n, vec3<i32>(0), still);
    n = select(n, max(n, vec3<i32>(1)), axis_mask(axis));
    let dims = (*w).dims;
    let cell = (*w).cell + (*w).dir * n;
    if (any(cell < vec3<i32>(0)) || any(cell >= dims)) {
        return false;
    }
    var texel = (*w).texel + (*w).dir * n;
    texel = select(texel, texel - dims, texel >= dims);
    texel = select(texel, texel + dims, texel < vec3<i32>(0));
    (*w).cell = cell;
    (*w).texel = texel;
    (*w).t_max += vec3<f32>(n) * (*w).t_delta;
    (*w).t = e.x;
    (*w).axis = axis;
    return true;
}
