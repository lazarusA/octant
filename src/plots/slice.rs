//! The slicer's cut as the plot shaders read it (`shaders/common/slice.wgsl`),
//! and where the volume and point cloud box puts the slicer's fractions.

use bytemuck::{Pod, Zeroable};

/// The slicer's cut as every plot shader reads it (`Slice` of
/// `shaders/common/slice.wgsl`, built by `app::state::slicer::Slicer::uniform`):
/// `lo.w` the mode (0 off, 1 box, 2 planes); the box keeps `lo..=hi` per
/// axis; planes lie at `lo.xyz` (negative: none). Positions are fractions of
/// the plotted window in the renderer's own axis order.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct SliceUniform {
    pub lo: [f32; 4],
    pub hi: [f32; 4],
}

impl SliceUniform {
    pub const OFF: Self = Self {
        lo: [0.0; 4],
        hi: [1.0, 1.0, 1.0, 0.0],
    };
    /// `lo.w` of the box and planes modes.
    pub const BOX: f32 = 1.0;
    pub const PLANES: f32 = 2.0;
}

/// Relief below this world height counts as flat: a surface or sphere then
/// has no height axis to cut (the gizmo's, and the shaders' heights).
pub const FLAT_HEIGHT: f32 = 1e-3;

/// The axes the volume's unit box runs against the slicer's fractions: its
/// y runs up and its z toward the viewer (rows run top down and planes
/// front to back in the data).
pub const VOLUME_FLIP: [bool; 3] = [false, true, true];

/// Where fractions `f` of a volume or point cloud lie in the world, the box
/// centred on the origin with sizes `aspects` (`get_3d_aspect_ratio`).
pub fn volume_world(f: [f32; 3], aspects: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| {
        let unit = if VOLUME_FLIP[k] { 1.0 - f[k] } else { f[k] };
        (unit - 0.5) * aspects[k]
    })
}

/// The fractions at world point `p` of that box: `volume_world`'s inverse.
pub fn volume_fraction(p: [f32; 3], aspects: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| {
        let unit = p[k] / aspects[k].max(1e-6) + 0.5;
        if VOLUME_FLIP[k] { 1.0 - unit } else { unit }
    })
}

/// Whether a surface or sphere draws heights at all (its gizmo then has a
/// height axis): relief taller than `slice::FLAT_HEIGHT` in the world, as
/// `ui::hover` measures it (`height_span`, `radius_span`).
pub fn has_relief(
    scale: &crate::utils::height::HeightScale,
    sphere: bool,
    mode: u32,
    disp: f32,
) -> bool {
    let k = match (sphere, mode) {
        (true, 0) => return false,
        (true, _) => 0.4,
        (false, 1) => 0.6,
        (false, _) => 0.8,
    };
    let (low, high) = scale.bounds();
    k * disp * (high.max(0.0) - low.min(0.0)) >= FLAT_HEIGHT
}
