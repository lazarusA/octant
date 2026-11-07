//! The volume and point cloud hovers under the slicer, as their shaders draw
//! them: rays enter and leave the slicer's box, and in the planes mode read
//! the nearest plane drawing something there (`volume/planes.wgsl`).

use crate::app::{OctantApp, SliceMode, Slicer};
use crate::plots::slice::{volume_fraction, volume_world};
use crate::ui::hover::camera::Ray3D;
use crate::ui::hover::volume_sampler::VolumeSampler;

/// The slicer's box (the whole box without one) in the world coordinates of
/// a box centred on the origin, `aspects` its sizes (`slice::volume_world`).
pub fn cut_bounds(slice: &Slicer, (ax, ay, az): (f32, f32, f32)) -> ([f32; 3], [f32; 3]) {
    let (lo, hi) = match slice.mode {
        SliceMode::Box => (slice.lo, slice.hi),
        SliceMode::Off | SliceMode::Planes => ([0.0; 3], [1.0; 3]),
    };
    let (a, b) = (
        volume_world(lo, [ax, ay, az]),
        volume_world(hi, [ax, ay, az]),
    );
    (
        std::array::from_fn(|k| a[k].min(b[k])),
        std::array::from_fn(|k| a[k].max(b[k])),
    )
}

impl VolumeSampler<'_> {
    /// The planes mode's hit: the nearest shown plane `ray` meets within
    /// `span` whose voxel is drawn, as `(x, y, z, value)`.
    pub(super) fn plane_hit(
        &self,
        app: &OctantApp,
        ray: &Ray3D,
        (t_in, t_out): (f32, f32),
        (ax, ay, az): (f32, f32, f32),
    ) -> Option<(usize, usize, usize, f32)> {
        let slice = &app.plot_configs.slice;
        let aspects = [ax, ay, az];
        let mut best: Option<(f32, (usize, usize, usize, f32))> = None;
        for k in 0..3 {
            let Some(at) = slice.planes[k] else { continue };
            let world = volume_world([at; 3], aspects)[k];
            if ray.dir[k].abs() < 1e-6 {
                continue;
            }
            let t = (world - ray.origin[k]) / ray.dir[k];
            if t < t_in || t > t_out || best.is_some_and(|(b, _)| t >= b) {
                continue;
            }
            let p = [0, 1, 2].map(|i| ray.origin[i] + t * ray.dir[i]);
            let voxel = self.voxel_at(volume_fraction(p, aspects));
            if self.is_visible(app, voxel.3) {
                best = Some((t, voxel));
            }
        }
        best.map(|(_, hit)| hit)
    }

    /// The drawn voxel at fractions `f` and its value (a thinned cloud's
    /// cell under it, as drawn).
    fn voxel_at(&self, f: [f32; 3]) -> (usize, usize, usize, f32) {
        let index = |f: f32, n: usize| ((f * n as f32) as usize).min(n.saturating_sub(1));
        let (_, (gw, gh)) = self.drawn_cell(0, 0);
        let (cx, cy) = self.drawn_voxel(index(f[0], gw), index(f[1], gh));
        let cz = index(f[2], self.depth);
        (cx, cy, cz, self.sample_cell(cx, cy, cz))
    }
}
