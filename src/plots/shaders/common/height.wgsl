// Data heights shared by the surface, the sphere and their coastlines
// (CPU mirror: `utils::height::HeightScale`).

// Channels of a packed RGB composite value (`pack_rgb`), each in [0, 1].
fn unpack_composite_rgb(val: f32) -> vec3<f32> {
    let packed = u32(max(val, 0.0));
    return vec3<f32>(
        f32(packed & 0xFFu),
        f32((packed >> 8u) & 0xFFu),
        f32((packed >> 16u) & 0xFFu),
    ) / 255.0;
}

// Position of `val` in the color range; symmetric about zero when the range
// spans it.
fn range_height(val: f32, cmin: f32, cmax: f32) -> f32 {
    if (cmin < 0.0 && cmax > 0.0) {
        let max_abs = max(abs(cmin), abs(cmax));
        return clamp(val / max_abs, -1.0, 1.0);
    }
    return clamp((val - cmin) / max(cmax - cmin, 1e-6), 0.0, 1.0);
}

// Height of `val` before displacement scaling: its color range position, or
// for RGB composites the length of its color (black 0, white 1). `flip`
// puts the top value at zero and the others above it.
fn normalized_height(val: f32, cmin: f32, cmax: f32, composite: bool, flip: bool) -> f32 {
    if (val != val) {
        return 0.0;
    }
    var h = range_height(val, cmin, cmax);
    var top = range_height(cmax, cmin, cmax);
    if (composite) {
        h = length(unpack_composite_rgb(val)) / sqrt(3.0);
        top = 1.0;
    }
    return select(h, top - h, flip);
}

// Smooth-mode vertex of a heightfield quad: values sit at cell centers, so the
// quad of a cell spans from its center to the next one's. The first and last
// vertices of an axis stretch to its outer edges so the surface covers the
// grid, unless the axis wraps around (global longitudes), where the last
// quad joins the last center to the first (CPU mirror: `ui::hover::smooth`).
struct SmoothCorner {
    // Cell whose value the vertex carries.
    cell: u32,
    // Cell and fraction within it where the vertex sits; one cell past the
    // last on a wrapped axis.
    at_cell: u32,
    frac: f32,
};

fn smooth_corner(cell: u32, pos: f32, n: u32, wrap: bool) -> SmoothCorner {
    let count = max(n, 1u);
    let j = cell + u32(round(pos));
    if (wrap) {
        return SmoothCorner(j % count, j, 0.5);
    }
    let last = count - 1u;
    var frac = 0.5;
    if (j == 0u) {
        frac = 0.0;
    } else if (j >= last) {
        frac = 1.0;
    }
    let at = min(j, last);
    return SmoothCorner(at, at, frac);
}
