//! Which table cells a resting view can see, guessed on the CPU before
//! any ray feedback: rays every `RAY_SPACING` canvas pixels march the plotted
//! (coarse) volume with DVR's opacity and mark the cells they cross until
//! they turn opaque. Bricks over no marked cell are hidden behind matter
//! and never split, so the atlas goes to the visible shell.
//!
//! Meant to be conservative: the shader's opacity, halved (`SAFETY`), so
//! rays reach deeper than the GPU's; a cell smaller than the ray spacing on
//! screen has voxels far below the split threshold, so no ray needs to hit
//! it. A coarse level can still hide what finer data shows through, and
//! the GPU's ray feedback corrects that a frame later. DVR and opaque label
//! surfaces hide matter (the projections read every voxel).

use super::select::Placement;
use crate::data::VolumeData;
use crate::data::multiscale::bricks::{BRICK, CellBits, cell_index, table_cells};
use crate::plots::camera::OrbitCamera;
use crate::plots::common::PlotColorParams;
use crate::utils::colormap::apply_color_scale_cpu;
use crate::utils::colormap::eval::colormap_alpha;

/// Canvas pixels between rays.
pub const RAY_SPACING: u32 = 32;
/// Accumulated opacity past which a ray stops.
const OPAQUE: f32 = 0.95;
/// Factor on each sample's opacity: rays see further than drawn.
const SAFETY: f32 = 0.5;
/// DVR's reference step (`sample.wgsl` `REFERENCE_STEP`), on a unit box.
const REFERENCE_STEP: f32 = 1.0 / 64.0;

/// How DVR turns values into opacity (`dvr.wgsl` `dvr_sample`).
#[derive(Clone, Copy, Debug)]
pub struct Opacity {
    /// The plot's colors: range, scale and opacity curve.
    pub color: PlotColorParams,
    /// DVR's Density.
    pub density: f32,
    /// Transparency off: the first value in range is an opaque surface.
    pub opaque: bool,
}

impl PartialEq for Opacity {
    fn eq(&self, other: &Self) -> bool {
        bytemuck::bytes_of(&self.color) == bytemuck::bytes_of(&other.color)
            && self.density == other.density
            && self.opaque == other.opaque
    }
}

/// Steps of the color range the colormap's opacity is sampled at once per
/// guess (its curve sits behind a lock).
const ALPHA_STEPS: usize = 256;

impl Opacity {
    /// The colormap's opacity (and curve) over the color range.
    fn colormap_alphas(&self) -> [f32; ALPHA_STEPS] {
        let c = &self.color;
        std::array::from_fn(|k| {
            let s = (k as f32 + 0.5) / ALPHA_STEPS as f32;
            colormap_alpha(c.cmin + s * (c.cmax - c.cmin), c)
        })
    }

    /// Opacity of one sample of value `v`, `steps` reference steps long: the
    /// shader's (scaled position, Density, colormap opacity `alphas`).
    fn alpha(&self, v: f32, steps: f32, alphas: &[f32; ALPHA_STEPS]) -> f32 {
        let c = &self.color;
        if !(v >= c.cmin.min(c.cmax) && v <= c.cmax.max(c.cmin)) {
            return 0.0;
        }
        if self.opaque {
            return 1.0;
        }
        let t = apply_color_scale_cpu(v, c.cmin, c.cmax, c.scale_type, c.scale_param);
        let ramp = t
            .clamp(0.001, 1.0)
            .powf(1.0 / self.density.max(0.1))
            .clamp(0.01, 1.0);
        let at = (v - c.cmin) / (c.cmax - c.cmin);
        let k = ((at * ALPHA_STEPS as f32) as usize).min(ALPHA_STEPS - 1);
        let a = ramp * alphas[k] * SAFETY;
        1.0 - (1.0 - a.min(0.9999)).powf(steps)
    }
}

/// The table cells (`grid`, x fastest) rays from `camera` on a canvas of
/// `px` pixels reach through `volume` (the plotted level, drawn order)
/// placed by `place` with storage flipped along `flip`.
pub fn seen_cells(
    volume: &VolumeData,
    camera: &OrbitCamera,
    place: &Placement,
    flip: [bool; 3],
    grid: [u32; 3],
    px: [u32; 2],
    opacity: Opacity,
) -> CellBits {
    let mut seen = CellBits::new(table_cells(grid).unwrap_or(0));
    let alphas = opacity.colormap_alphas();
    let [nx, ny] = px.map(|p| (p / RAY_SPACING).clamp(2, 256));
    for j in 0..ny {
        for i in 0..nx {
            let clip = [
                (i as f32 + 0.5) / nx as f32 * 2.0 - 1.0,
                (j as f32 + 0.5) / ny as f32 * 2.0 - 1.0,
            ];
            let (origin, dir) = camera.ray(clip);
            march(
                volume,
                place,
                flip,
                grid,
                (origin, dir),
                (opacity, &alphas),
                &mut seen,
            );
        }
    }
    seen
}

/// Marks the cells one ray crosses until it turns opaque.
fn march(
    volume: &VolumeData,
    place: &Placement,
    flip: [bool; 3],
    grid: [u32; 3],
    (origin, dir): ([f32; 3], [f32; 3]),
    (opacity, alphas): (Opacity, &[f32; ALPHA_STEPS]),
    seen: &mut CellBits,
) {
    let half = place.aspect.map(|a| a * 0.5);
    let Some((t0, t1)) = box_hit(origin, dir, half) else {
        return;
    };
    let n = [volume.width, volume.height, volume.depth];
    // One step per plotted voxel along the box's finest axis.
    let step = (0..3)
        .map(|i| place.aspect[i] / n[i].max(1) as f32)
        .fold(f32::MAX, f32::min);
    let steps = step / REFERENCE_STEP;
    let (mut t, mut acc) = (t0.max(0.0), 0.0);
    while t < t1 && acc < OPAQUE {
        let p: [f32; 3] = std::array::from_fn(|k| origin[k] + t * dir[k]);
        // Drawn (texture) fractions: x right, rows and planes top-down.
        let unit: [f32; 3] =
            std::array::from_fn(|k| (p[k] / place.aspect[k].max(1e-9) + 0.5).clamp(0.0, 1.0));
        let tex = [unit[0], 1.0 - unit[1], 1.0 - unit[2]];
        let voxel: [usize; 3] =
            std::array::from_fn(|k| ((tex[k] * n[k] as f32) as usize).min(n[k].saturating_sub(1)));
        let cell: [u32; 3] = std::array::from_fn(|k| {
            let s = if flip[k] { 1.0 - tex[k] } else { tex[k] } as f64;
            let q = place.lo[k] + s * (place.hi[k] - place.lo[k]);
            ((q / BRICK as f64) as u32).min(grid[k].saturating_sub(1))
        });
        seen.set(cell_index(grid, cell));
        let at = (voxel[2] * n[1] + voxel[1]) * n[0] + voxel[0];
        let a = opacity.alpha(
            volume.values.get(at).copied().unwrap_or(f32::NAN),
            steps,
            alphas,
        );
        acc += (1.0 - acc) * a;
        t += step;
    }
}

/// Where the ray enters and leaves the box of half sizes `half` at the
/// origin.
fn box_hit(origin: [f32; 3], dir: [f32; 3], half: [f32; 3]) -> Option<(f32, f32)> {
    let (mut t0, mut t1) = (f32::NEG_INFINITY, f32::INFINITY);
    for k in 0..3 {
        if dir[k].abs() < 1e-12 {
            if origin[k].abs() > half[k] {
                return None;
            }
            continue;
        }
        let (a, b) = (
            (-half[k] - origin[k]) / dir[k],
            (half[k] - origin[k]) / dir[k],
        );
        t0 = t0.max(a.min(b));
        t1 = t1.min(a.max(b));
    }
    (t0 <= t1 && t1 > 0.0).then_some((t0, t1))
}
