//! Typed configuration bundles for each visualization plot type.

/// Configuration for 1D line profile and multi-series curves.
#[derive(Clone, Debug, PartialEq)]
pub struct LinePlotConfig {
    pub profile_dim_idx: usize,
    pub profile_slice_idx: usize,
    pub all_series: bool,
    pub line_color: [f32; 4],
    pub use_custom_color: bool,
    pub show_lines: bool,
    pub show_points: bool,
    pub point_size: f32,
}

impl Default for LinePlotConfig {
    fn default() -> Self {
        Self {
            profile_dim_idx: 0,
            profile_slice_idx: 0,
            all_series: false,
            line_color: [0.2, 0.65, 1.0, 1.0],
            use_custom_color: true,
            show_lines: true,
            show_points: false,
            point_size: 6.0,
        }
    }
}

/// Configuration for 3D volumetric raymarching (DVR and isosurfaces).
#[derive(Clone, Debug, PartialEq)]
pub struct VolumePlotConfig {
    pub opacity: f32,
    /// Volume raymarching samples per voxel crossed by each ray.
    pub quality: f32,
    pub transparency: bool,
    pub lighting: bool,
    pub attenuation: f32,
    pub algorithm: u32,
    /// Where the opaque volume draws its surface.
    pub surface: VolumeSurface,
    /// The Value surface's value; NaN until Value is first chosen, which sets
    /// it from the data.
    pub isovalue: f32,
    pub z_scale: f32,
}

impl VolumePlotConfig {
    /// The Value surface's value within `(lo, hi)` (the data's extent): its
    /// middle while `isovalue` lies outside (NaN until chosen, or left from
    /// another variable).
    pub fn isovalue_in(&self, (lo, hi): (f32, f32)) -> f32 {
        if (lo..=hi).contains(&self.isovalue) {
            self.isovalue
        } else {
            0.5 * (lo + hi)
        }
    }

    /// Whether DVR draws its Value surface: opaque, Surface Value, not a
    /// composite (`composite`), which draws its brightness.
    pub fn draws_value_surface(&self, composite: bool) -> bool {
        self.algorithm == 0
            && !self.transparency
            && self.surface == VolumeSurface::Value
            && !composite
    }
}

/// Where an opaque DVR volume draws its surface (`VolumeUniforms::surface`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VolumeSurface {
    /// Where values enter the color range.
    #[default]
    Range,
    /// Where values reach `VolumePlotConfig::isovalue`.
    Value,
}

impl Default for VolumePlotConfig {
    fn default() -> Self {
        Self {
            opacity: 3.0,
            quality: 1.0,
            transparency: true,
            lighting: false,
            attenuation: 0.0,
            algorithm: 0,
            surface: VolumeSurface::Range,
            isovalue: f32::NAN,
            z_scale: 1.0,
        }
    }
}

/// Configuration for 3D surface mesh and spherical projections.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshPlotConfig {
    pub surface_displacement: f32,
    pub surface_mode: u32,
    /// Heights measured down from the top value (`HeightScale::flip`).
    pub surface_flip_height: bool,
    pub sphere_displacement: f32,
    pub sphere_mode: u32,
    pub sphere_flip_height: bool,
    /// How many cells large grids draw (`MeshDetail::budget`).
    pub detail: MeshDetail,
}

impl MeshPlotConfig {
    /// `(mode, displacement, flip_height)` of the sphere or the surface.
    pub fn heights(&self, sphere: bool) -> (u32, f32, bool) {
        if sphere {
            (
                self.sphere_mode,
                self.sphere_displacement,
                self.sphere_flip_height,
            )
        } else {
            (
                self.surface_mode,
                self.surface_displacement,
                self.surface_flip_height,
            )
        }
    }
}

impl Default for MeshPlotConfig {
    fn default() -> Self {
        Self {
            surface_displacement: 0.3,
            surface_mode: 0,
            surface_flip_height: false,
            sphere_displacement: 0.3,
            sphere_mode: 0,
            sphere_flip_height: false,
            detail: MeshDetail::default(),
        }
    }
}

/// How many cells a surface or sphere draws at most, against the canvas's
/// pixels; grids with more are thinned (`MatrixData::decimated`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeshDetail {
    /// Half the canvas's pixels: more cells would mostly be under a pixel.
    Screen,
    Double,
    Quadruple,
    /// Every cell that fits one storage buffer.
    #[default]
    Full,
}

impl MeshDetail {
    pub const ALL: [Self; 4] = [Self::Screen, Self::Double, Self::Quadruple, Self::Full];

    /// The most cells drawn on a canvas of `pixels`.
    pub fn budget(self, pixels: usize) -> usize {
        let screen = pixels / 2;
        match self {
            Self::Screen => screen,
            Self::Double => screen.saturating_mul(2),
            Self::Quadruple => screen.saturating_mul(4),
            Self::Full => usize::MAX,
        }
    }
}

/// Configuration for 3D point cloud billboard rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct PointCloudPlotConfig {
    /// Point width as a fraction of the closest gap between points (1 = touching).
    pub point_size: f32,
}

impl Default for PointCloudPlotConfig {
    fn default() -> Self {
        Self { point_size: 0.8 }
    }
}

/// Consolidated plot configuration bundles.
#[derive(Clone, Debug, PartialEq)]
pub struct PlotConfigs {
    pub line: LinePlotConfig,
    pub volume: VolumePlotConfig,
    pub mesh: MeshPlotConfig,
    pub point_cloud: PointCloudPlotConfig,
    /// Whether 3D translucent plots draw without depth writes (OIT/NoDepthWrite).
    pub plot_transparency: bool,
}

impl Default for PlotConfigs {
    fn default() -> Self {
        Self {
            line: LinePlotConfig::default(),
            volume: VolumePlotConfig::default(),
            mesh: MeshPlotConfig::default(),
            point_cloud: PointCloudPlotConfig::default(),
            plot_transparency: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VolumePlotConfig;

    #[test]
    fn isovalue_stays_within_the_data() {
        let at = |isovalue| VolumePlotConfig {
            isovalue,
            ..Default::default()
        };
        // Unset, or left from a variable with other values: the middle.
        assert_eq!(at(f32::NAN).isovalue_in((200.0, 300.0)), 250.0);
        assert_eq!(at(0.5).isovalue_in((200.0, 300.0)), 250.0);
        assert_eq!(at(220.0).isovalue_in((200.0, 300.0)), 220.0);
    }
}
