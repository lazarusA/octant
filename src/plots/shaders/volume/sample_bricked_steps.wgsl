// How a ray steps through bricks (`sample_bricked.wgsl`): table cells and
// 8^3 sub-blocks that cannot show anything are skipped, coarser bricks are
// sampled on their own stride from where the ray enters them, and each
// sample stands for the steps it spans, cut where its cell or sub-block
// ends (and the ray). Samples read the brick of the cell they are in
// (`blocks_scalar`), found once per cell.

// A fixed-step ray's samples in level-0 positions (`a + i * d`, `count` of
// them), the table cell the last one asked about, whether it may draw and
// how many steps each of its samples stands for; the cell's brick (its slot
// and first texel, where its interior starts, its cells' size, whether it
// misses voxels) and the sub-block last asked about.
struct SampleBlocks {
    a: vec3<f32>,
    d: vec3<f32>,
    count: i32,
    cell: vec3<i32>,
    open: bool,
    stride: i32,
    resident: bool,
    missing: bool,
    slot: u32,
    slot0: vec3<f32>,
    origin: vec3<f32>,
    factor: vec3<f32>,
    sub: i32,
    sub_open: bool,
    // Where the stride starts (the first step in the cell or sub-block, or
    // where a skip landed), the first step past the cell and past the
    // current stretch (the cell or sub-block, whichever ends first), and
    // the sample last taken and the one before it.
    i0: i32,
    cell_end: i32,
    end: i32,
    at: i32,
    prev: i32,
};

fn sample_blocks(ray: Ray) -> SampleBlocks {
    let a = level0_position(ray.start);
    let d = level0_position(ray.start + ray.step) - a;
    let none = vec3<f32>(0.0);
    return SampleBlocks(a, d, ray.count, vec3<i32>(-1), true, 1, false, false, 0u, none, none,
        vec3<f32>(1.0), -1, true, 0, 0, 0, -1, -1);
}

// Enters table cell `cell` at step `i`: its brick, stride and end.
fn enter_cell(s: ptr<function, SampleBlocks>, i: i32, cell: vec3<i32>) {
    (*s).cell = cell;
    mark_seen(cell);
    let e = textureLoad(brick_table, cell, 0);
    (*s).resident = e.w != 0u;
    (*s).stride = 1;
    (*s).i0 = i;
    (*s).cell_end = i32(ceil(cell_exit((*s).a, (*s).d, cell)));
    (*s).end = (*s).cell_end;
    (*s).sub = -1;
    if (!(*s).resident) {
        return;
    }
    let lv = bricked.levels[min(e.w - 1u, 15u)];
    (*s).stride = brick_stride(lv, (*s).d);
    (*s).slot = slot_index(e.xyz);
    (*s).slot0 = vec3<f32>(e.xyz) * STORED;
    (*s).factor = lv.factor.xyz;
    (*s).origin = lv.offset.xyz + cell_brick(cell, lv) * BRICK * lv.factor.xyz;
    (*s).missing = brick_ranges[(*s).slot].z > 0.5;
}

// The sample to take next from `i` in an open cell: past the sub-block of
// sample `i` when it cannot show what `kind` asks (never past the cell).
fn next_open_sub(s: ptr<function, SampleBlocks>, i: i32, kind: u32, q: vec3<f32>) -> i32 {
    let sub_size = (*s).factor * (BRICK / SUB_SIDE);
    let sb = clamp(vec3<i32>(floor((q - (*s).origin) / sub_size)), vec3<i32>(0), vec3<i32>(7));
    let index = sb.x + 8 * (sb.y + 8 * sb.z);
    if (index != (*s).sub) {
        (*s).sub = index;
        let range = brick_subs[(*s).slot * 512u + u32(index)];
        (*s).sub_open = block_open(range, kind, 0.0);
        // Samples start again in each sub-block and stand for no step past
        // it (its exit, or the cell's): none counts for an empty sub-block
        // beside it.
        let lo = (*s).origin + vec3<f32>(sb) * sub_size;
        let exit = i32(ceil(box_exit((*s).a, (*s).d, lo, lo + sub_size)));
        (*s).i0 = i;
        (*s).end = min((*s).cell_end, exit);
    }
    if ((*s).sub_open) {
        return i;
    }
    return max(i + 1, (*s).end);
}

// Sample parameter where `a + t * d` leaves the box `lo..hi`.
fn box_exit(a: vec3<f32>, d: vec3<f32>, lo: vec3<f32>, hi: vec3<f32>) -> f32 {
    let edge = select(lo, hi, d > vec3<f32>(0.0));
    let t = select(vec3<f32>(1e30), (edge - a) / d, abs(d) > vec3<f32>(1e-9));
    return min(t.x, min(t.y, t.z));
}

// Samples per step of a brick of level `lv`: steps are as long as the
// finest level drawn wants, so a coarser brick takes one sample in
// `stride`, still `quality` per its voxel.
fn brick_stride(lv: BrickLevel, d: vec3<f32>) -> i32 {
    let f = lv.factor.xyz;
    let per_voxel = min(f.x, min(f.y, f.z)) / max(length(d), 1e-6);
    return clamp(i32(floor(per_voxel / max(uniforms.quality, 0.05))), 1, 64);
}

// The table cell holding level-0 position `q`.
fn sample_cell(q: vec3<f32>) -> vec3<i32> {
    let grid = vec3<i32>(bricked.grid.xyz);
    return clamp(vec3<i32>(floor(q / BRICK)), vec3<i32>(0), grid - 1);
}

// The sample to take next from `i`: past table cells whose brick cannot
// show what `kind` asks, and on the stride of coarser bricks.
fn next_open_sample(s: ptr<function, SampleBlocks>, i: i32, kind: u32) -> i32 {
    let q = (*s).a + (*s).d * f32(i);
    let cell = sample_cell(q);
    if (any(cell != (*s).cell)) {
        enter_cell(s, i, cell);
        (*s).open = (*s).resident && block_open(brick_ranges[(*s).slot], kind, 0.0);
    }
    if (!(*s).open) {
        return max(i + 1, (*s).cell_end);
    }
    let sub = next_open_sub(s, i, kind, q);
    if (sub != i) {
        // The stride starts again where the skip lands.
        (*s).i0 = sub;
        return sub;
    }
    return on_stride(s, i);
}

// The sample to take next from `i` in a mode that never skips by value
// (the classic modes): on the stride of coarser bricks, marking the cells
// the ray reaches.
fn next_strided_sample(s: ptr<function, SampleBlocks>, i: i32) -> i32 {
    let cell = sample_cell((*s).a + (*s).d * f32(i));
    if (any(cell != (*s).cell)) {
        enter_cell(s, i, cell);
    }
    return on_stride(s, i);
}

// Sample parameter where `a + t * d` leaves table cell `cell`.
fn cell_exit(a: vec3<f32>, d: vec3<f32>, cell: vec3<i32>) -> f32 {
    let lo = vec3<f32>(cell) * BRICK;
    return box_exit(a, d, lo, lo + BRICK);
}

// `i` when it lies on the stride from `i0` (and it is taken), else the
// next step that does: a coarse brick's samples start where the ray enters
// it, not on global multiples of the stride.
fn on_stride(s: ptr<function, SampleBlocks>, i: i32) -> i32 {
    let k = (*s).stride;
    let off = i - (*s).i0;
    if (off % k != 0) {
        // Never past the stretch: the next one starts its own stride.
        return min((*s).i0 + (off / k + 1) * k, max((*s).end, i + 1));
    }
    (*s).prev = (*s).at;
    (*s).at = i;
    return i;
}

// Steps the sample last taken stands for: the stride, cut at the end of its
// cell or sub-block (and of the ray) so no step is counted twice, for empty
// space or past the ray.
fn sample_span(s: ptr<function, SampleBlocks>) -> f32 {
    return f32(clamp(min((*s).end, (*s).count) - (*s).at, 1, (*s).stride));
}

// Where a surface search before sample `i` starts: the sample taken before
// it (a coarse brick's stride back), the ray's entry for the first.
fn previous_taken(s: ptr<function, SampleBlocks>, ray: Ray, i: i32) -> vec3<f32> {
    return select(sample_pos(ray, (*s).prev), ray.entry, (*s).prev < 0 || i == 0);
}

// Atlas texel position of unit-box position `p` in the brick of the
// cell the ray is in (resident).
fn blocks_texel(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> vec3<f32> {
    return brick_texel((*s).slot0, (*s).origin, (*s).factor, level0_position(p));
}

// `sample_scalar` at sample position `p`, read from the cell's brick
// without looking it up again.
fn blocks_scalar(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> f32 {
    if (!(*s).resident) {
        return MISSING;
    }
    return filtered_value(blocks_texel(s, p), (*s).missing);
}

// `sample_exact` at sample position `p`, from the cell's brick.
fn blocks_exact(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> f32 {
    if (!(*s).resident) {
        return MISSING;
    }
    let t = vec3<u32>(floor(blocks_texel(s, p)));
    return stored_value(textureLoad(volume_values, t, 0).r);
}

// `sample_composite` at sample position `p`, from the cell's brick.
fn blocks_composite(s: ptr<function, SampleBlocks>, p: vec3<f32>) -> vec4<f32> {
    if (!(*s).resident) {
        return vec4<f32>(0.0);
    }
    let a = blocks_texel(s, p);
    return fetch_trilinear(a / vec3<f32>(textureDimensions(volume_values)));
}
