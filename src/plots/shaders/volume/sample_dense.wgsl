// Dense volume reads: unit-box positions to the volume texture's
// coordinates (ring-buffer shift applied), trilinear and exact reads,
// validity. The bricked variant (`sample_bricked.wgsl`) defines the same
// functions over the brick atlas.

fn volume_dims() -> vec3<f32> {
    return vec3<f32>(textureDimensions(volume_values));
}

fn volume_shift() -> vec3<u32> {
    return vec3<u32>(uniforms.shift_x, uniforms.shift_y, uniforms.shift_z);
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

// The texel `t` as stored: scalar in `r`, composite color in `rgba`.
fn texel_value(t: vec3<u32>) -> vec4<f32> {
    return textureLoad(volume_values, t, 0);
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

// The sample to take next from `i` in a mode that never skips by value
// (the classic modes): `i` here (`sample_bricked.wgsl` strides coarse
// bricks).
fn next_strided_sample(s: ptr<function, SampleBlocks>, i: i32) -> i32 {
    return i;
}

// Steps each taken sample stands for: always one here (`sample_bricked.wgsl`
// takes fewer in coarse bricks).
fn sample_span(s: ptr<function, SampleBlocks>) -> f32 {
    return 1.0;
}

// Where a surface search before sample `i` starts: the sample before it
// (`sample_bricked_steps.wgsl` steps coarse bricks by more).
fn previous_taken(s: ptr<function, SampleBlocks>, ray: Ray, i: i32) -> vec3<f32> {
    return previous_pos(ray, i);
}

// `sample_scalar`, `sample_exact` and `sample_composite` at a ray's sample
// position `p` (the bricked variant reads the brick its step found).
fn blocks_scalar(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> f32 {
    return sample_scalar(p);
}

fn blocks_exact(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> f32 {
    return sample_exact(p);
}

fn blocks_composite(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> vec4<f32> {
    return sample_composite(p);
}
