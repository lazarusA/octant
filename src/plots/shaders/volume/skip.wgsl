// Empty-space skipping: rays jump over bricks that cannot contribute to the
// current mode, landing on the next sample of the same lattice, so skipping
// never changes the image.

// Brick edge in voxels (`bricks::BRICK`).
const BRICK: f32 = 8.0;

// What a brick must hold to matter (see `brick_matters`).
const SKIP_NONE: u32 = 0u;  // never skip
const SKIP_DVR: u32 = 1u;   // DVR: visible in-range values, or a visible clip/NaN color
const SKIP_RANGE: u32 = 2u; // values inside [cmin, cmax]
const SKIP_VALID: u32 = 3u; // any valid value, or a visible NaN color
const SKIP_ABOVE: u32 = 4u; // a value above `level`
const SKIP_BELOW: u32 = 5u; // a value below `level`

fn sample_pos(ray: Ray, i: i32) -> vec3<f32> {
    return ray.start + f32(i) * ray.step;
}

// Sample before `i` (the ray entry for the first): where a surface search
// starts, as every skipped or evaluated sample before `i` was outside it.
fn previous_pos(ray: Ray, i: i32) -> vec3<f32> {
    return select(sample_pos(ray, i - 1), ray.entry, i == 0);
}

// Logical texel coordinate of `p` (rows top-down, planes front to back).
fn logical_texel(p: vec3<f32>) -> vec3<f32> {
    return logical_coord(p) * volume_dims();
}

// Brick of logical texel `l`, ring-buffer shift applied, as (brick, wrapped):
// `wrapped` marks axes whose shifted coordinate passed the volume's end.
fn brick_of(l: vec3<f32>) -> array<vec3<f32>, 2> {
    let dims = volume_dims();
    let shifted = clamp(l, vec3<f32>(0.0), dims - 0.001) + vec3<f32>(volume_shift());
    let wrapped = select(vec3<f32>(0.0), dims, shifted >= dims);
    return array<vec3<f32>, 2>(floor((shifted - wrapped) / BRICK), wrapped);
}

fn brick_matters(s: vec4<f32>, kind: u32, level: f32) -> bool {
    let c = uniforms.color;
    let valid = s.x <= s.y;
    let nan_shown = s.z > 0.5 && c.use_nan_color == 1u && c.nan_color.a > 0.0;
    switch kind {
        case 1u: {
            let low = c.use_lowclip == 1u && c.lowclip_color.a > 0.0 && s.x < c.cmin;
            let high = c.use_highclip == 1u && c.highclip_color.a > 0.0 && s.y > c.cmax;
            // Every in-range sample is at least faintly visible.
            return nan_shown || (valid && ((s.y >= c.cmin && s.x <= c.cmax) || low || high));
        }
        case 2u: { return valid && s.y >= c.cmin && s.x <= c.cmax; }
        case 3u: { return valid || nan_shown; }
        case 4u: { return valid && s.y > level; }
        case 5u: { return valid && s.x < level; }
        default: { return true; }
    }
}

// Sample index along the ray where the brick holding sample `i` ends.
fn brick_exit(ray: Ray, i: i32) -> f32 {
    let dims = volume_dims();
    let a = logical_texel(ray.start);
    let b = vec3<f32>(ray.step.x, -ray.step.y, -ray.step.z) * dims;
    let brick = brick_of(a + f32(i) * b);
    // Brick bounds back in logical texels (the last brick may be partial).
    let offset = brick[1] - vec3<f32>(volume_shift());
    let lo = brick[0] * BRICK + offset;
    let hi = min(brick[0] * BRICK + BRICK, dims) + offset;
    let bound = select(lo, hi, b > vec3<f32>(0.0));
    let k = select(vec3<f32>(1e30), (bound - a) / b, abs(b) > vec3<f32>(1e-12));
    return min(k.x, min(k.y, k.z));
}

// First sample index from `i` on whose brick matters for `kind`: each empty
// brick is crossed in one jump.
fn skip_empty(ray: Ray, i_in: i32, kind: u32, level: f32) -> i32 {
    var i = i_in;
    if (kind == SKIP_NONE) {
        return i;
    }
    for (var guard = 0; guard < 64 && i < ray.count; guard = guard + 1) {
        let brick = brick_of(logical_texel(sample_pos(ray, i)));
        let stats = textureLoad(volume_bricks, vec3<i32>(brick[0]), 0);
        if (brick_matters(stats, kind, level)) {
            return i;
        }
        i = max(i + 1, i32(ceil(brick_exit(ray, i))));
    }
    return i;
}
