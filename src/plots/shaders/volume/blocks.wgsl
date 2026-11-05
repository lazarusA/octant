// Empty-space skipping (`blocks.rs`): rays jump over blocks of 8^3 stored
// voxels that cannot show anything. A block's texel holds the min and max of
// the valid values in it and its neighbors and 1 where any is missing, so
// trilinear taps and gradients near it are covered.

const BLOCK_SHIFT: u32 = 3u;
const BLOCK_EDGE: i32 = 8;

// What a ray asks of a block: a value its colors draw (DVR and label
// surfaces, NaN and clip colors included), a value at or above `isovalue`,
// or a value above a running maximum (MIP).
const OPEN_DRAWN: u32 = 0u;
const OPEN_VALUE: u32 = 1u;
const OPEN_ABOVE: u32 = 2u;

fn block_open(b: vec4<f32>, kind: u32, maximum: f32) -> bool {
    if (kind == OPEN_VALUE) {
        return b.y >= uniforms.isovalue;
    }
    if (kind == OPEN_ABOVE) {
        return b.y > maximum;
    }
    let c = uniforms.color;
    let lo = select(c.cmin, -3.4e38, c.use_lowclip == 1u);
    let hi = select(c.cmax, 3.4e38, c.use_highclip == 1u);
    return (b.z > 0.5 && c.use_nan_color == 1u) || (b.x <= hi && b.y >= lo);
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
    n[axis] = max(n[axis], 1);
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

// A fixed-step ray's samples in logical texel positions (`a + i * d`) and
// the block the last one asked about, for DVR to skip blocks.
struct SampleBlocks {
    a: vec3<f32>,
    d: vec3<f32>,
    block: vec3<i32>,
    open: bool,
};

fn sample_blocks(ray: Ray) -> SampleBlocks {
    let a = walk_texel_pos(ray.start);
    return SampleBlocks(a, walk_texel_pos(ray.start + ray.step) - a, vec3<i32>(-1), true);
}

// The sample to take next from `i`: `i` when its block may hold what
// `kind` asks, else the first sample past that block.
fn next_open_sample(s: ptr<function, SampleBlocks>, i: i32, kind: u32) -> i32 {
    let dims = vec3<i32>(textureDimensions(volume_values));
    let p = (*s).a + (*s).d * f32(i);
    let cell = clamp(vec3<i32>(floor(p)), vec3<i32>(0), dims - 1);
    var texel = cell + vec3<i32>(volume_shift());
    texel = select(texel, texel - dims, texel >= dims);
    let block = texel >> vec3<u32>(BLOCK_SHIFT);
    if (any(block != (*s).block)) {
        (*s).block = block;
        (*s).open = block_open(textureLoad(volume_blocks, block, 0), kind, 0.0);
    }
    if ((*s).open) {
        return i;
    }
    let e = block_exit((*s).a, (*s).d, cell, texel, dims);
    return max(i + 1, i32(ceil(e.x)));
}
