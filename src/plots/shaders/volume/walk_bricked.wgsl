// Voxel walk over bricks (`sample_bricked.wgsl`): `walk.wgsl`'s functions for MIP and label surfaces. The walk crosses
// the table cells a ray's segment passes, in order, and inside each the
// voxels of the level resident there, each once, so nothing is stepped
// over at any level. Positions are level-0 storage positions of the plotted
// window. While moving (`volume_dims` coarser than a brick's level), a
// brick's voxels are walked on that coarser grid, reading the voxel at each
// coarse cell's center.

const WALK_NEVER: f32 = 1e30;
// Parameter gap under which a voxel face counts as its table cell's face.
const WALK_TIE: f32 = 1e-5;

struct Walk {
    // The segment, `a + t * span` in level-0 storage positions.
    a: vec3<f32>,
    span: vec3<f32>,
    // +1 or -1 along each axis, 0 where the segment runs parallel to it.
    dir: vec3<i32>,
    // Table cell, its entry (w: level + 1, 0 when nothing is resident),
    // where the segment leaves it and the axis it leaves through.
    cell: vec3<i32>,
    entry: vec4<u32>,
    cell_exit: f32,
    cell_axis: i32,
    // The grid walked in the cell: cell size and origin (level-0 units),
    // and the level's voxel size.
    size: vec3<f32>,
    origin: vec3<f32>,
    factor: vec3<f32>,
    // Atlas texel of the level's voxel 0 through this cell's brick, and the
    // brick's first stored texel.
    base: vec3<i32>,
    slot0: vec3<i32>,
    // Current grid cell and its atlas texel.
    voxel: vec3<i32>,
    texel: vec3<i32>,
    t_max: vec3<f32>,
    t_delta: vec3<f32>,
    // Parameter where the walk entered the current voxel, and the axis of
    // the face it entered through (-1 for the segment's first voxel).
    t: f32,
    axis: i32,
};

fn axis_mask(axis: i32) -> vec3<bool> {
    return vec3<i32>(0, 1, 2) == vec3<i32>(axis);
}

// The first axis of `m` holding its minimum: x, then y, then z on ties.
fn first_min_axis(m: vec3<f32>) -> i32 {
    let t = min(min(m.x, m.y), m.z);
    return select(select(2, 1, m.y == t), 0, m.x == t);
}

// Atlas texel of the brick voxel at the current grid cell's center.
fn walk_texel_at(w: Walk) -> vec3<i32> {
    let center = (vec3<f32>(w.voxel) + 0.5) * w.size;
    let v = vec3<i32>(floor(center / w.factor));
    return clamp(w.base + v, w.slot0, w.slot0 + i32(STORED) - 1);
}

// Enters table cell `cell` at segment parameter `t`.
fn walk_enter(w: ptr<function, Walk>, t: f32, cell: vec3<i32>) {
    let ahead = (*w).span > vec3<f32>(0.0);
    let parallel = (*w).dir == vec3<i32>(0);
    let safe = select((*w).span, vec3<f32>(1.0), parallel);
    let face = (vec3<f32>(cell) + select(vec3<f32>(0.0), vec3<f32>(1.0), ahead)) * BRICK;
    let tc = select((face - (*w).a) / safe, vec3<f32>(WALK_NEVER), parallel);
    (*w).cell = cell;
    mark_seen(cell);
    (*w).cell_exit = min(min(tc.x, tc.y), tc.z);
    (*w).cell_axis = first_min_axis(tc);
    let e = textureLoad(brick_table, cell, 0);
    (*w).entry = e;
    var size = vec3<f32>(BRICK);
    var origin = vec3<f32>(0.0);
    var factor = size;
    var slot0 = vec3<i32>(0);
    var base = vec3<i32>(0);
    if (e.w != 0u) {
        let lv = bricked.levels[min(e.w - 1u, 15u)];
        factor = lv.factor.xyz;
        origin = lv.offset.xyz;
        let b = cell_brick(cell, lv);
        slot0 = vec3<i32>(e.xyz) * i32(STORED);
        base = slot0 + i32(APRON) - vec3<i32>(b) * i32(BRICK);
        let coarse = bricked.span.xyz / volume_dims();
        size = select(factor, coarse, coarse > factor * 1.01);
    }
    (*w).size = size;
    (*w).origin = origin;
    (*w).factor = factor;
    (*w).slot0 = slot0;
    (*w).base = base;
    // Nudged along the ray, so a start on a face picks the voxel ahead.
    let q = (*w).a + (*w).span * t + vec3<f32>((*w).dir) * size * 0.01 - origin;
    let v = vec3<i32>(floor(q / size));
    (*w).voxel = v;
    let next = (vec3<f32>(v) + select(vec3<f32>(0.0), vec3<f32>(1.0), ahead)) * size + origin;
    (*w).t_max = select((next - (*w).a) / safe, vec3<f32>(WALK_NEVER), parallel);
    (*w).t_delta = select(size / abs(safe), vec3<f32>(WALK_NEVER), parallel);
    (*w).texel = walk_texel_at(*w);
    (*w).t = t;
}

// The walk along the ray's whole segment, from its entry to its exit.
fn walk_begin(ray: Ray) -> Walk {
    var w: Walk;
    w.a = level0_position(ray.entry);
    w.span = level0_position(ray.entry + ray.step * f32(ray.count)) - w.a;
    w.dir = vec3<i32>(sign(w.span));
    let grid = vec3<i32>(bricked.grid.xyz);
    let start = w.a + vec3<f32>(w.dir) * 0.01;
    let cell = clamp(vec3<i32>(floor(start / BRICK)), vec3<i32>(0), grid - 1);
    walk_enter(&w, 0.0, cell);
    w.axis = -1;
    return w;
}

// Most voxels any walk visits: level-0 voxels and table cells crossed.
fn walk_limit(w: Walk) -> i32 {
    let n = bricked.span.xyz;
    let g = vec3<f32>(bricked.grid.xyz);
    return i32(n.x + n.y + n.z) + 2 * i32(g.x + g.y + g.z) + 3;
}

// Parameter where the walk leaves the current voxel.
fn walk_exit(w: Walk) -> f32 {
    return min(min(min(w.t_max.x, w.t_max.y), w.t_max.z), w.cell_exit);
}

// Moves into the next table cell; returns false once the segment or the
// table ends.
fn walk_next_cell(w: ptr<function, Walk>) -> bool {
    let t = (*w).cell_exit;
    if (t >= 1.0) {
        return false;
    }
    let axis = (*w).cell_axis;
    let cell = (*w).cell + select(vec3<i32>(0), (*w).dir, axis_mask(axis));
    if (any(cell < vec3<i32>(0)) || any(cell >= vec3<i32>(bricked.grid.xyz))) {
        return false;
    }
    walk_enter(w, t, cell);
    (*w).axis = axis;
    return true;
}

// Moves to the next voxel; returns false once the segment ends.
fn walk_step(w: ptr<function, Walk>) -> bool {
    let m = (*w).t_max;
    let t = min(min(m.x, m.y), m.z);
    if ((*w).cell_exit <= t + WALK_TIE) {
        return walk_next_cell(w);
    }
    if (t >= 1.0) {
        return false;
    }
    let axis = first_min_axis(m);
    let on = axis_mask(axis);
    (*w).t_max = select(m, m + (*w).t_delta, on);
    (*w).voxel += select(vec3<i32>(0), (*w).dir, on);
    (*w).texel = walk_texel_at(*w);
    (*w).t = t;
    (*w).axis = axis;
    return true;
}

// Whether the current voxel is missing: no brick resident, or NaN (a
// composite's missing voxels are transparent instead).
fn walk_missing(w: Walk) -> bool {
    if (w.entry.w == 0u) {
        return true;
    }
    if (is_composite()) {
        return false;
    }
    return atlas_nan(textureLoad(volume_values, w.texel, 0).r);
}

// The current voxel: a composite's color, else its value in every lane.
fn walk_texel(w: Walk) -> vec4<f32> {
    if (w.entry.w == 0u) {
        return vec4<f32>(MISSING);
    }
    return texel_value(vec3<u32>(w.texel));
}

// Unit-box normal of the face the walk entered the current voxel through
// (facing back along the ray), or `entry_normal` for the first voxel.
fn walk_face_normal(w: Walk, entry_normal: vec3<f32>) -> vec3<f32> {
    if (w.axis < 0) {
        return entry_normal;
    }
    // Storage runs opposite to the logical order where flipped.
    let flip = select(vec3<f32>(1.0), vec3<f32>(-1.0), bricked.flip.xyz != vec3<u32>(0u));
    let n = select(vec3<f32>(0.0), -vec3<f32>(w.dir) * flip, axis_mask(w.axis));
    // Logical rows run top-down and planes front to back (`logical_coord`).
    return n * vec3<f32>(1.0, -1.0, -1.0);
}

// Whether the walk's table cell may hold what `kind` asks, read only when
// the walk enters another cell (`last`, `open` carry the previous answer).
fn walk_block_open(
    w: Walk,
    last: ptr<function, vec3<i32>>,
    open: ptr<function, bool>,
    kind: u32,
    maximum: f32,
) -> bool {
    if (any(w.cell != *last)) {
        *last = w.cell;
        let e = w.entry;
        *open = e.w != 0u && block_open(brick_ranges[slot_index(e.xyz)], kind, maximum);
    }
    return *open;
}

// Moves the walk past its table cell.
fn walk_skip_block(w: ptr<function, Walk>) -> bool {
    return walk_next_cell(w);
}
