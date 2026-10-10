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
    pub isovalue: f32,
    pub isorange: f32,
    pub z_scale: f32,
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
            isovalue: 50.0,
            isorange: 5.0,
            z_scale: 1.0,
        }
    }
}

/// Configuration for 3D surface mesh and spherical projections.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshPlotConfig {
    pub surface_displacement: f32,
    pub surface_mode: u32,
    pub sphere_displacement: f32,
    pub sphere_mode: u32,
}

impl Default for MeshPlotConfig {
    fn default() -> Self {
        Self {
            surface_displacement: 0.3,
            surface_mode: 0,
            sphere_displacement: 0.3,
            sphere_mode: 0,
        }
    }
}

/// Configuration for 3D point cloud billboard rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct PointCloudPlotConfig {
    pub point_size: f32,
}

impl Default for PointCloudPlotConfig {
    fn default() -> Self {
        Self { point_size: 0.02 }
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
