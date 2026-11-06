//! The voxels a hover ray crosses, in order: the CPU twin of the shaders'
//! `walk.wgsl` (3D DDA), so MIP and label surface hovers name the voxel the
//! shader draws.

use crate::app::OctantApp;
use crate::ui::hover::camera::Ray3D;
use crate::ui::hover::volume_sampler::VolumeSampler;
use crate::utils::colormap::evaluate_color_cpu;

/// A hovered voxel: logical indices and its value.
pub type VoxelHit = (usize, usize, usize, f32);

/// Calls `visit(voxel, t_in, t_out)` for every voxel of a `dims` grid the
/// segment `a + t (b - a)`, t in [0, 1], crosses, in order, until it returns
/// false. Positions are logical texel positions (texel k spans [k, k + 1)).
pub fn walk_voxels(
    a: [f32; 3],
    b: [f32; 3],
    dims: [usize; 3],
    mut visit: impl FnMut([usize; 3], f32, f32) -> bool,
) {
    if dims.contains(&0) {
        return;
    }
    let span = [0, 1, 2].map(|i| b[i] - a[i]);
    let mut cell = [0usize; 3];
    let mut dir = [0i64; 3];
    let mut t_max = [f32::INFINITY; 3];
    let mut t_delta = [f32::INFINITY; 3];
    for i in 0..3 {
        let ahead = span[i] > 0.0;
        if span[i] != 0.0 {
            dir[i] = if ahead { 1 } else { -1 };
        }
        // Nudged along the ray, so a start on a face picks the voxel ahead
        // (by the shader's `walk_begin` amount, so both pick the same one).
        let start = a[i] + dir[i] as f32 * 1e-3;
        cell[i] = (start.floor().max(0.0) as usize).min(dims[i] - 1);
        if span[i] != 0.0 {
            let face = cell[i] as f32 + if ahead { 1.0 } else { 0.0 };
            t_max[i] = (face - a[i]) / span[i];
            t_delta[i] = (1.0 / span[i]).abs();
        }
    }
    let mut t = 0.0;
    loop {
        let axis = (0..3).fold(0, |m, i| if t_max[i] < t_max[m] { i } else { m });
        if !visit(cell, t, t_max[axis].min(1.0)) || t_max[axis] >= 1.0 {
            return;
        }
        t = t_max[axis];
        t_max[axis] += t_delta[axis];
        let next = cell[axis] as i64 + dir[axis];
        if next < 0 || next >= dims[axis] as i64 {
            return;
        }
        cell[axis] = next as usize;
    }
}

impl VolumeSampler<'_> {
    /// The voxel MIP or the label surface (`is_mip` false) shows along the
    /// ray's part `t` inside the box (half-extents `aspects`), on the walk
    /// their shaders take.
    pub(super) fn walk_hit(
        &self,
        app: &OctantApp,
        ray: &Ray3D,
        (t_start, t_end): (f32, f32),
        (ax, ay, az): (f32, f32, f32),
        is_mip: bool,
    ) -> Option<VoxelHit> {
        // Logical texel position: rows top-down, planes front to back.
        let texel = |t: f32| {
            let p = [0, 1, 2].map(|i| ray.origin[i] + t * ray.dir[i]);
            [
                (p[0] / ax.max(1e-4) + 0.5) * self.width as f32,
                (0.5 - p[1] / ay.max(1e-4)) * self.height as f32,
                (0.5 - p[2] / az.max(1e-4)) * self.depth as f32,
            ]
        };
        let (a, b) = (texel(t_start), texel(t_end));
        if is_mip {
            self.mip_hit(app, a, b)
        } else {
            self.label_hit(app, a, b)
        }
    }

    /// The voxel the MIP shader shows along segment `a`..`b` (logical texel
    /// positions): the largest value it considers, attenuated as it is.
    pub(super) fn mip_hit(&self, app: &OctantApp, a: [f32; 3], b: [f32; 3]) -> Option<VoxelHit> {
        let color = &app.layers.base.color;
        let attenuation = app.plot_configs.volume.attenuation;
        let (cmin, cmax) = (color.range_min, color.range_max);
        let highclip_visible = color.highclip_color[3] > 0.0;
        let range = (cmax - cmin).max(1e-4);
        let (mut maximum, mut best, mut sum) = (f32::NEG_INFINITY, None, 0.0);
        let dims = [self.width, self.height, self.depth];
        walk_voxels(a, b, dims, |[x, y, z], t_in, t_out| {
            let v = self.sample_cell(x, y, z);
            if v.is_nan() || v.abs() > 1e30 {
                return true;
            }
            let mut attenuated = v;
            if attenuation > 0.0 {
                sum += ((v - cmin) / range).clamp(0.0, 1.0) * (t_out - t_in);
                attenuated = v * (-attenuation * sum).exp();
            }
            if (v <= cmax || highclip_visible) && attenuated > maximum {
                maximum = attenuated;
                best = Some((x, y, z, v));
            }
            // Past cmax every larger value shows the same color.
            attenuation > 0.0 || best.is_none_or(|(_, _, _, top)| top <= cmax)
        });
        best
    }

    /// The voxel of the label surface the ray meets first along `a`..`b`:
    /// the first one drawn (`label_color` in `rgba.wgsl`): visible, and not
    /// hidden by the layer's opacity, curve or a transparent clip color.
    pub(super) fn label_hit(&self, app: &OctantApp, a: [f32; 3], b: [f32; 3]) -> Option<VoxelHit> {
        let params = app.get_color_params(&app.layers.base);
        let mut hit = None;
        let dims = [self.width, self.height, self.depth];
        walk_voxels(a, b, dims, |[x, y, z], _, _| {
            let v = self.sample_cell(x, y, z);
            if self.is_visible(app, v) && evaluate_color_cpu(v, &params).a() > 0 {
                hit = Some((x, y, z, v));
            }
            hit.is_none()
        });
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::walk_voxels;

    fn visited(a: [f32; 3], b: [f32; 3], dims: [usize; 3]) -> Vec<[usize; 3]> {
        let mut cells = vec![];
        walk_voxels(a, b, dims, |c, _, _| {
            cells.push(c);
            true
        });
        cells
    }

    #[test]
    fn a_straight_ray_visits_every_voxel_once() {
        let cells = visited([0.5, 0.5, 0.0], [0.5, 0.5, 600.0], [1, 1, 600]);
        assert_eq!(cells.len(), 600);
        assert!(cells.iter().enumerate().all(|(i, c)| c[2] == i));
    }

    #[test]
    fn a_diagonal_ray_moves_one_face_at_a_time() {
        let cells = visited([0.0, 0.2, 7.9], [8.0, 6.7, 0.1], [8, 8, 8]);
        assert_eq!(cells.first(), Some(&[0, 0, 7]));
        assert_eq!(cells.last(), Some(&[7, 6, 0]));
        // Neighbors share a face: one axis moves by one.
        for pair in cells.windows(2) {
            let moved: usize = (0..3).map(|i| pair[0][i].abs_diff(pair[1][i])).sum();
            assert_eq!(moved, 1, "{pair:?}");
        }
        assert_eq!(cells.len(), 7 + 6 + 7 + 1);
    }

    #[test]
    fn the_walk_stops_when_asked() {
        let mut n = 0;
        walk_voxels([0.0; 3], [8.0, 0.0, 0.0], [8, 1, 1], |_, _, _| {
            n += 1;
            n < 3
        });
        assert_eq!(n, 3);
    }
}
