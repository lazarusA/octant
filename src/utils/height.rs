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

    /// Lowest and highest heights `normalized` can return.
    pub fn bounds(&self) -> (f32, f32) {
        if self.composite {
            return (0.0, 1.0);
        }
        let (low, high) = (self.range_height(self.cmin), self.range_height(self.cmax));
        if self.flip {
            (0.0, high - low)
        } else {
            (low, high)
        }
    }

    /// The value whose height is `h` (inside `bounds`), the inverse of
    /// `normalized`; `None` for composites, whose heights are colors.
    pub fn value_at(&self, h: f32) -> Option<f32> {
        if self.composite {
            return None;
        }
        let (cmin, cmax) = (self.cmin, self.cmax);
        let h = if self.flip {
            self.range_height(cmax) - h
        } else {
            h
        };
        Some(if cmin < 0.0 && cmax > 0.0 {
            h * cmin.abs().max(cmax.abs())
        } else {
            cmin + h * (cmax - cmin)
        })
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

/// World height per unit of normalized height (`HeightScale::normalized`)
/// of the surface (`sphere` false) or the sphere drawn in `mode`, before the
/// displacement strength, as surface.wgsl and sphere.wgsl draw them (pinned
/// by `shaders_use_the_same_height_factors`): the surface's steps 0.6, its
/// other modes 0.8, the sphere 0.4 of the radius, none in its mode 0.
pub fn height_factor(sphere: bool, mode: u32) -> f32 {
    match (sphere, mode) {
        (true, 0) => 0.0,
        (true, _) => 0.4,
        (false, 1) => 0.6,
        (false, _) => 0.8,
    }
}

/// Height of `val` on the surface mesh, matching surface.wgsl.
#[inline]
pub fn surface_height(val: f32, scale: &HeightScale, surface_mode: u32, disp: f32) -> f32 {
    if !val.is_finite() {
        return 0.0;
    }
    scale.normalized(val) * height_factor(false, surface_mode) * disp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shaders_use_the_same_height_factors() {
        let surface = include_str!("../plots/shaders/surface.wgsl");
        let sphere = include_str!("../plots/shaders/sphere.wgsl");
        let coastline = include_str!("../plots/shaders/coastline_3d.wgsl");
        for text in [surface, coastline] {
            assert!(
                text.contains("* 0.6 * uniforms.displacement_strength")
                    || text.contains("* 0.6 * coastline_uniforms.displacement_strength")
            );
            assert!(
                text.contains("* 0.8 * uniforms.displacement_strength")
                    || text.contains("* 0.8 * coastline_uniforms.displacement_strength")
            );
        }
        assert!(sphere.contains("h * 0.4 * uniforms.displacement_strength"));
        assert!(coastline.contains("* 0.4 * coastline_uniforms.displacement_strength"));
        assert!(surface.contains("select(0.8, 0.6, uniforms.mode == 1u)"));
        assert!(sphere.contains("let height_scale = 0.4 * uniforms.displacement_strength;"));
        let factors = [
            (false, 0),
            (false, 1),
            (false, 2),
            (true, 0),
            (true, 1),
            (true, 3),
        ];
        let got = factors.map(|(s, m)| height_factor(s, m));
        assert_eq!(got, [0.8, 0.6, 0.8, 0.0, 0.4, 0.4]);
    }
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
        assert_eq!(s.bounds(), (0.0, 1.5));
        assert_eq!(scale(-50.0, 100.0, false, false).bounds(), (-0.5, 1.0));
        assert_eq!(scale(0.0, 16_777_215.0, true, true).bounds(), (0.0, 1.0));
    }
}
