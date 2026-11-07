//! The slicer's cut as the plot shaders read it (`shaders/common/slice.wgsl`).

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
    pub const BOX: f32 = 1.0;
    pub const PLANES: f32 = 2.0;

    /// The cut with the axes in `flip` reversed (fraction `f` read as
    /// `1 - f`), for renderers whose axes run the other way.
    pub fn flipped(self, flip: [bool; 3]) -> Self {
        let mut out = self;
        for k in (0..3).filter(|&k| flip[k]) {
            if self.lo[3] == Self::PLANES {
                out.lo[k] = if self.lo[k] < 0.0 {
                    -1.0
                } else {
                    1.0 - self.lo[k]
                };
            } else {
                (out.lo[k], out.hi[k]) = (1.0 - self.hi[k], 1.0 - self.lo[k]);
            }
        }
        out
    }

    /// The box's w range replaced by `range` (a surface's or sphere's
    /// normalized heights instead of fractions).
    pub fn with_w(self, range: (f32, f32)) -> Self {
        let mut out = self;
        if self.lo[3] == Self::BOX {
            (out.lo[2], out.hi[2]) = range;
        }
        out
    }
}
