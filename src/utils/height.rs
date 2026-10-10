//! Data heights of the surface and sphere, matching `shaders/common/height.wgsl`.

use crate::data::slicing::composite::unpack_rgb;

/// How values map to heights before displacement scaling.
#[derive(Clone, Copy, Debug)]
pub struct HeightScale {
    pub cmin: f32,
    pub cmax: f32,
    /// Values are packed RGB composite colors, raised by their color's length.
    pub composite: bool,
    /// The top value sits at zero and the others above it.
    pub flip: bool,
}

impl HeightScale {
    /// Height of `val`: its color range position (symmetric about zero when
    /// the range spans it), or for composites its color's length (black 0,
    /// white 1); flipped from the top value when `flip`.
    pub fn normalized(&self, val: f32) -> f32 {
        if val.is_nan() {
            return 0.0;
        }
        let (h, top) = if self.composite {
            let (r, g, b) = unpack_rgb(val.max(0.0));
            ((r * r + g * g + b * b).sqrt() / (255.0 * 3f32.sqrt()), 1.0)
        } else {
            (self.range_height(val), self.range_height(self.cmax))
        };
        if self.flip { top - h } else { h }
    }

    fn range_height(&self, val: f32) -> f32 {
        let (cmin, cmax) = (self.cmin, self.cmax);
        if cmin < 0.0 && cmax > 0.0 {
            let max_abs = cmin.abs().max(cmax.abs());
            (val / max_abs).clamp(-1.0, 1.0)
        } else {
            ((val - cmin) / (cmax - cmin).max(1e-6)).clamp(0.0, 1.0)
        }
    }
}

/// Height of `val` on the surface mesh, matching surface.wgsl.
#[inline]
pub fn surface_height(val: f32, scale: &HeightScale, surface_mode: u32, disp: f32) -> f32 {
    if !val.is_finite() {
        return 0.0;
    }
    let mult = match surface_mode {
        1 => 0.6, // Flat Steps
        _ => 0.8, // Smooth Terrain (0) and 3D Lego Cubes (2)
    };
    scale.normalized(val) * mult * disp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::slicing::composite::pack_rgb;

    fn scale(cmin: f32, cmax: f32, composite: bool, flip: bool) -> HeightScale {
        HeightScale {
            cmin,
            cmax,
            composite,
            flip,
        }
    }

    #[test]
    fn surface_height_follows_range() {
        let s = scale(0.0, 100.0, false, false);
        assert!((surface_height(50.0, &s, 0, 1.0) - 0.4).abs() < 1e-4);
        assert_eq!(surface_height(f32::NAN, &s, 0, 1.0), 0.0);
    }

    #[test]
    fn composite_height_is_color_length() {
        // The range of packed values must not matter.
        let s = scale(0.0, 16_777_215.0, true, false);
        assert_eq!(s.normalized(pack_rgb(0.0, 0.0, 0.0)), 0.0);
        assert!((s.normalized(pack_rgb(255.0, 255.0, 255.0)) - 1.0).abs() < 1e-6);
        let red = s.normalized(pack_rgb(255.0, 0.0, 0.0));
        let blue = s.normalized(pack_rgb(0.0, 0.0, 255.0));
        assert!((red - 1.0 / 3f32.sqrt()).abs() < 1e-6);
        assert_eq!(red, blue);
    }

    #[test]
    fn flip_puts_top_at_zero() {
        let s = scale(0.0, 255.0, false, true);
        assert_eq!(s.normalized(255.0), 0.0);
        assert_eq!(s.normalized(0.0), 1.0);

        let white = scale(0.0, 16_777_215.0, true, true);
        assert!(white.normalized(pack_rgb(255.0, 255.0, 255.0)).abs() < 1e-6);
        assert_eq!(white.normalized(pack_rgb(0.0, 0.0, 0.0)), 1.0);
    }

    #[test]
    fn flip_spanning_zero_stays_above_plane() {
        let s = scale(-50.0, 100.0, false, true);
        assert_eq!(s.normalized(100.0), 0.0);
        assert!((s.normalized(-50.0) - 1.5).abs() < 1e-6);
    }
}
