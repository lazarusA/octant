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
// A value at or above the color range's low end (MinIP: lower values are
// never its minimum without a visible low clip color; higher ones draw).
const OPEN_AT_LEAST: u32 = 3u;

fn block_open(b: vec4<f32>, kind: u32, maximum: f32) -> bool {
    if (kind == OPEN_VALUE) {
        return b.y >= uniforms.isovalue;
    }
    if (kind == OPEN_ABOVE) {
        return b.y > maximum;
    }
    if (kind == OPEN_AT_LEAST) {
        return b.y >= uniforms.color.cmin;
    }
    let c = uniforms.color;
    let lo = select(c.cmin, -3.4e38, c.use_lowclip == 1u);
    let hi = select(c.cmax, 3.4e38, c.use_highclip == 1u);
    return (b.z > 0.5 && c.use_nan_color == 1u) || (b.x <= hi && b.y >= lo);
}
