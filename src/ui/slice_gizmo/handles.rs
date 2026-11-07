//! The box mode's handles: the cut box's edges and corners, each named by
//! the faces it borders.

use super::guide::Guide;
use super::space::Space;
use crate::app::Slicer;

/// Most handles a box has: 12 edges and 8 corners.
pub const MAX_HANDLES: usize = 20;

/// Which face of an axis a handle sits on: `Some(false)` its `lo` face,
/// `Some(true)` its `hi` one, `None` the axis the edge runs along (or an
/// axis the space does not cut).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    pub sides: [Option<bool>; 3],
}

impl Guide for Handle {
    fn point(&self, slice: &Slicer, t: f32) -> [f32; 3] {
        std::array::from_fn(|k| match self.sides[k] {
            Some(true) => slice.hi[k],
            Some(false) => slice.lo[k],
            None => slice.lo[k] + (slice.hi[k] - slice.lo[k]) * t,
        })
    }

    /// Any face it borders facing the camera (the outer shell for a flat
    /// globe or surface).
    fn visible(&self, space: &Space, slice: &Slicer, t: f32) -> bool {
        let p = self.point(slice, t);
        if space.dims == 2 {
            return space.faces_camera(p, 2, true);
        }
        (0..3).any(|k| self.sides[k].is_some_and(|hi| space.faces_camera(p, k, hi)))
    }

    fn is_corner(&self, dims: usize) -> bool {
        self.sides[..dims].iter().all(Option::is_some)
    }
}

/// Every handle of a space of `dims` axes, corners first.
pub fn all(dims: usize) -> ([Handle; MAX_HANDLES], usize) {
    let mut out = [Handle { sides: [None; 3] }; MAX_HANDLES];
    let mut n = 0;
    let corners = 1usize << dims;
    for bits in 0..corners {
        out[n].sides = std::array::from_fn(|k| (k < dims).then_some(bits >> k & 1 == 1));
        n += 1;
    }
    for along in 0..dims {
        for bits in 0..corners / 2 {
            let mut sides = [None; 3];
            for (j, k) in (0..dims).filter(|&k| k != along).enumerate() {
                sides[k] = Some(bits >> j & 1 == 1);
            }
            out[n].sides = sides;
            n += 1;
        }
    }
    (out, n)
}
