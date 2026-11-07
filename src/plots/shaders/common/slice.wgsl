// The slicer's cut (`plots::common::SliceUniform`, from
// `app::state::slicer`): `lo.w` the mode (0 off, 1 box, 2 planes); the box
// keeps `lo..=hi` per axis; planes lie at `lo.xyz` (negative: none).
// Positions are fractions of the plotted window in the shader's axis order.

struct Slice {
    lo: vec4<f32>,
    hi: vec4<f32>,
};

const SLICE_BOX: f32 = 1.0;
const SLICE_PLANES: f32 = 2.0;

// Whether `f` lies in the box (always without one).
fn slice_box_keeps(s: Slice, f: vec3<f32>) -> bool {
    return s.lo.w != SLICE_BOX || (all(f >= s.lo.xyz) && all(f <= s.hi.xyz));
}

// Whether the cell centred at `c`, `h` its half sizes, holds a shown plane
// (always without planes).
fn slice_planes_keep(s: Slice, c: vec3<f32>, h: vec3<f32>) -> bool {
    if (s.lo.w != SLICE_PLANES) {
        return true;
    }
    return any((s.lo.xyz >= vec3<f32>(0.0)) & (abs(c - s.lo.xyz) <= h));
}
