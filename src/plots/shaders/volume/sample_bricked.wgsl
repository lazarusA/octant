// Bricked volume reads (`plots/volume/bricks.rs`): the same functions as `sample_dense.wgsl`, over bricks of several
// levels. A unit-box position becomes a level-0 position of the plotted
// window; its table cell (`BRICK` level-0 cells a side) names the finest
// resident brick's level and atlas slot; the brick holding the cell's
// center at that level is the one in the slot, sampled trilinearly inside
// its stored cube (interior and a one-cell apron), never past it.

const BRICK: f32 = 64.0;
const APRON: f32 = 1.0;
const STORED: f32 = 66.0;

struct BrickLevel {
    // Level-0 cells per cell, and where cell 0 starts, along x, y, z.
    factor: vec4<f32>,
    offset: vec4<f32>,
    // Bricks along x, y, z.
    count: vec4<u32>,
};

struct Bricked {
    // The plotted window's level-0 start and extent along x, y, z.
    lo: vec4<f32>,
    span: vec4<f32>,
    // Non-zero where storage runs opposite to the logical (drawn) order.
    flip: vec4<u32>,
    // Table cells along x, y, z.
    grid: vec4<u32>,
    // Stored value = atlas value * y + x.
    value: vec4<f32>,
    // Atlas slots along x, y, z.
    slots: vec4<u32>,
    levels: array<BrickLevel, 16>,
};

@group(0) @binding(5)
var<uniform> bricked: Bricked;

// Per table cell: the atlas slot (xyz) and level + 1 (w, 0 when none).
@group(0) @binding(6)
var brick_table: texture_3d<u32>;

// Per atlas slot (x fastest): its brick's min and max valid value, and 1
// where it has missing voxels, as `volume_blocks` holds them per block.
@group(0) @binding(7)
var<storage, read> brick_ranges: array<vec4<f32>>;

// Per atlas slot, per 8^3 sub-block of its brick (x fastest, 512 a slot):
// min, max and missing as `brick_ranges`, for skipping inside a brick.
@group(0) @binding(9)
var<storage, read> brick_subs: array<vec4<f32>>;

const SUB_SIDE: f32 = 8.0;

// One bit per table cell (x fastest) a ray entered before it stopped: ray
// feedback for the LOD selection (`plots/volume/feedback.rs`).
@group(0) @binding(8)
var<storage, read_write> brick_seen: array<atomic<u32>>;

// Only resting frames record (`record_seen`), and a bit already set is not
// written again: many rays enter each cell.
fn mark_seen(cell: vec3<i32>) {
    if (uniforms.record_seen == 0u) {
        return;
    }
    let g = bricked.grid;
    let i = u32(cell.x) + g.x * (u32(cell.y) + g.y * u32(cell.z));
    let bit = 1u << (i & 31u);
    if ((atomicLoad(&brick_seen[i >> 5u]) & bit) == 0u) {
        atomicOr(&brick_seen[i >> 5u], bit);
    }
}

// Voxels across the window at the finest level drawn: the ray's step.
fn volume_dims() -> vec3<f32> {
    return max(vec3<f32>(f32(uniforms.width), f32(uniforms.height), f32(uniforms.depth)), vec3<f32>(1.0));
}

// Level-0 position (storage order) of unit-box position `p`.
fn level0_position(p: vec3<f32>) -> vec3<f32> {
    let l = clamp(logical_coord(p), vec3<f32>(0.0), vec3<f32>(1.0));
    let s = select(l, 1.0 - l, bricked.flip.xyz != vec3<u32>(0u));
    return bricked.lo.xyz + s * bricked.span.xyz;
}

// The brick of level `lv` holding table cell `cell`'s center (its index
// along x, y, z): the one the table's slot holds (`data/multiscale/
// bricks.rs` `cell_brick`).
fn cell_brick(cell: vec3<i32>, lv: BrickLevel) -> vec3<f32> {
    let center = (vec3<f32>(cell) + 0.5) * BRICK;
    let last = vec3<f32>(lv.count.xyz) - 1.0;
    return clamp(floor((center - lv.offset.xyz) / (lv.factor.xyz * BRICK)), vec3<f32>(0.0), last);
}

// The atlas slot index (x fastest) of slot coordinates `e`.
fn slot_index(e: vec3<u32>) -> u32 {
    return e.x + bricked.slots.x * (e.y + bricked.slots.y * e.z);
}

// Atlas texel position of level-0 position `q` (w 1, 2 when its brick has
// missing voxels), or w 0 where no brick is resident.
fn atlas_position(q: vec3<f32>) -> vec4<f32> {
    let grid = vec3<i32>(bricked.grid.xyz);
    let cell = clamp(vec3<i32>(floor(q / BRICK)), vec3<i32>(0), grid - 1);
    let e = textureLoad(brick_table, cell, 0);
    if (e.w == 0u) {
        return vec4<f32>(0.0);
    }
    let lv = bricked.levels[min(e.w - 1u, 15u)];
    let origin = lv.offset.xyz + cell_brick(cell, lv) * BRICK * lv.factor.xyz;
    let missing = brick_ranges[slot_index(e.xyz)].z > 0.5;
    let at = brick_texel(vec3<f32>(e.xyz) * STORED, origin, lv.factor.xyz, q);
    return vec4<f32>(at, select(1.0, 2.0, missing));
}

// Atlas texel position of level-0 position `q` in the brick whose interior
// starts at `origin` (cells `factor` level-0 units wide) in the slot whose
// first texel is `slot0`: inside its stored cube, never past it.
fn brick_texel(slot0: vec3<f32>, origin: vec3<f32>, factor: vec3<f32>, q: vec3<f32>) -> vec3<f32> {
    let at = (q - origin) / factor + APRON;
    return slot0 + clamp(at, vec3<f32>(0.5), vec3<f32>(STORED - 0.5));
}

// Whether atlas value `v` is a missing voxel's NaN (tested on the bits:
// NaN comparisons may be folded away).
fn atlas_nan(v: f32) -> bool {
    return (bitcast<u32>(v) & 0x7fffffffu) > 0x7f800000u;
}

// Missing voxels are stored as NaN (`filtered_value` keeps them out of
// filtering).
fn stored_value(atlas: f32) -> f32 {
    if (atlas_nan(atlas)) {
        return MISSING;
    }
    return atlas * bricked.value.y + bricked.value.x;
}

// Trilinear value at atlas texel position `a`. In a brick with `missing`
// voxels the eight taps are weighed by validity, as the dense volume's
// validity texture does: missing where valid taps weigh under half, else
// the mean of the valid ones (hardware filtering would spread NaN).
fn filtered_value(a: vec3<f32>, missing: bool) -> f32 {
    if (!missing) {
        return stored_value(fetch_trilinear(a / vec3<f32>(textureDimensions(volume_values))).r);
    }
    let c = a - 0.5;
    let base = floor(c);
    let f = c - base;
    let slot0 = floor(a / STORED) * STORED;
    var sum = 0.0;
    var weight = 0.0;
    for (var k = 0u; k < 8u; k = k + 1u) {
        let o = vec3<f32>(f32(k & 1u), f32((k >> 1u) & 1u), f32((k >> 2u) & 1u));
        let t = clamp(base + o, slot0, slot0 + STORED - 1.0);
        let w3 = mix(1.0 - f, f, o);
        let v = textureLoad(volume_values, vec3<i32>(t), 0).r;
        if (!atlas_nan(v)) {
            let w = w3.x * w3.y * w3.z;
            sum += w * v;
            weight += w;
        }
    }
    if (weight < 0.5) {
        return MISSING;
    }
    return stored_value(sum / weight);
}

// `exact_texel`'s answer where nothing is resident.
const NO_TEXEL: u32 = 0xffffffffu;

// Trilinear scalar at `p`, or MISSING where nothing is resident.
fn sample_scalar(p: vec3<f32>) -> f32 {
    let a = atlas_position(level0_position(p));
    if (a.w == 0.0) {
        return MISSING;
    }
    return filtered_value(a.xyz, a.w > 1.5);
}

// Trilinear composite color at `p` (alpha the brightest channel, a
// composite's `Rgba8Unorm` atlas), zero where nothing is resident or
// missing.
fn sample_composite(p: vec3<f32>) -> vec4<f32> {
    let a = atlas_position(level0_position(p));
    if (a.w == 0.0) {
        return vec4<f32>(0.0);
    }
    return fetch_trilinear(a.xyz / vec3<f32>(textureDimensions(volume_values)));
}

// Atlas texel of the voxel holding `p`, or `NO_TEXEL` where nothing is
// resident (`texel_missing`).
fn exact_texel(p: vec3<f32>) -> vec3<u32> {
    let a = atlas_position(level0_position(p));
    if (a.w == 0.0) {
        return vec3<u32>(NO_TEXEL);
    }
    return vec3<u32>(floor(a.xyz));
}

// Atlas texel `t` (`exact_texel`'s, resident): a composite's color, else
// its value in every lane.
fn texel_value(t: vec3<u32>) -> vec4<f32> {
    let v = textureLoad(volume_values, t, 0);
    if (is_composite()) {
        return v;
    }
    return vec4<f32>(stored_value(v.r));
}

// Missing where nothing is resident, or a scalar voxel stored as NaN (a
// composite's missing voxels are transparent colors instead).
fn texel_missing(t: vec3<u32>) -> bool {
    return t.x == NO_TEXEL || (!is_composite() && atlas_nan(textureLoad(volume_values, t, 0).r));
}

// Exact value of the voxel containing `p`, or MISSING.
fn sample_exact(p: vec3<f32>) -> f32 {
    let t = exact_texel(p);
    if (texel_missing(t)) {
        return MISSING;
    }
    return stored_value(textureLoad(volume_values, t, 0).r);
}
