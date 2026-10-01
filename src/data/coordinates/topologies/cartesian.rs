//! Cartesian regular grid topologies (Global and Regional).

use crate::data::coordinates::topology::GridTopology;
use crate::plots::PlotType;

static CARTESIAN_PLOT_TYPES: &[PlotType] = &[
    PlotType::Heatmap,
    PlotType::Line,
    PlotType::Surface,
    PlotType::Volume,
    PlotType::Sphere,
    PlotType::PointCloud,
];

/// Cartesian regular grid with uniform spacing.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CartesianTopology {
    /// Full global spherical grid spanning ~360° lon and ~180° lat.
    #[default]
    Global,
    /// Regional bounding box with uniform step size.
    Regional {
        lon_bounds: (f32, f32),
        lat_bounds: (f32, f32),
    },
}

impl CartesianTopology {
    #[inline]
    pub fn default_supported_plots() -> &'static [PlotType] {
        CARTESIAN_PLOT_TYPES
    }

    #[inline]
    fn is_regional(&self) -> bool {
        matches!(self, Self::Regional { .. })
    }
}

impl GridTopology for CartesianTopology {
    fn name(&self) -> &'static str {
        match self {
            Self::Global => "Cartesian Regular (Global)",
            Self::Regional { .. } => "Cartesian Regular (Regional)",
        }
    }

    fn spatial_rank(&self) -> usize {
        2
    }

    fn supported_plot_types(&self) -> &'static [PlotType] {
        CARTESIAN_PLOT_TYPES
    }

    fn data_aspect_ratio(&self, width: usize, height: usize) -> f32 {
        let w = width.max(1) as f32;
        let h = height.max(1) as f32;
        if self.is_global() {
            2.0
        } else {
            (w / h).clamp(0.1, 10.0)
        }
    }

    fn surface_uv_to_cell(&self, u: f32, v: f32, width: usize, height: usize) -> (usize, usize) {
        self.cell_center_from_norm(u.clamp(0.0, 1.0), v.clamp(0.0, 1.0), width, height)
    }

    fn lon_lat_to_cell(
        &self,
        lon_rad: f32,
        lat_rad: f32,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        let w = width.max(1);
        let h = height.max(1);

        match self {
            Self::Global => {
                let u = ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI))
                    .clamp(0.0, 1.0);
                let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
                let px = ((u * w as f32).floor() as usize).min(w.saturating_sub(1));
                let py = ((v * h as f32).floor() as usize).min(h.saturating_sub(1));
                Some((px, py))
            }
            Self::Regional { .. } => {
                let [lon_min, lon_max] = self.lon_bounds_rad();
                let [lat_min, lat_max] = self.lat_bounds_rad();

                let mut query_lon = lon_rad;
                if lon_max > std::f32::consts::PI && query_lon < 0.0 {
                    query_lon += 2.0 * std::f32::consts::PI;
                }

                if query_lon < lon_min - 0.05
                    || query_lon > lon_max + 0.05
                    || lat_rad < lat_min - 0.05
                    || lat_rad > lat_max + 0.05
                {
                    return None;
                }

                let span_lon = (lon_max - lon_min).abs().max(1e-6);
                let span_lat = (lat_max - lat_min).abs().max(1e-6);
                let u = ((query_lon - lon_min) / span_lon).clamp(0.0, 1.0);
                let v = ((lat_max - lat_rad) / span_lat).clamp(0.0, 1.0);

                let px = ((u * w as f32).floor() as usize).min(w.saturating_sub(1));
                let py = ((v * h as f32).floor() as usize).min(h.saturating_sub(1));
                Some((px, py))
            }
        }
    }

    fn cell_center_norm(&self, px: usize, py: usize, width: usize, height: usize) -> (f32, f32) {
        let w = width.max(1);
        let h = height.max(1);
        ((px as f32 + 0.5) / w as f32, (py as f32 + 0.5) / h as f32)
    }

    fn cell_center_surface_xz(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
        aspect: f32,
    ) -> (f32, f32) {
        let (u_c, v_c) = self.cell_center_norm(px, py, width, height);
        ((2.0 * u_c - 1.0) * aspect, 2.0 * v_c - 1.0)
    }

    fn cell_center_lon_lat_rad(
        &self,
        px: usize,
        py: usize,
        width: usize,
        height: usize,
    ) -> (f32, f32) {
        let w = width.max(1);
        let h = height.max(1);

        match self {
            Self::Global => {
                let (u_c, v_c) = self.cell_center_norm(px, py, width, height);
                let lon_rad = (u_c - 0.5) * 2.0 * std::f32::consts::PI;
                let lat_rad = (0.5 - v_c) * std::f32::consts::PI;
                (lon_rad, lat_rad)
            }
            Self::Regional { .. } => {
                let [lon_min, lon_max] = self.lon_bounds_rad();
                let [lat_min, lat_max] = self.lat_bounds_rad();
                let lon_rad = if w > 1 {
                    lon_min + (px as f32 / (w - 1) as f32) * (lon_max - lon_min)
                } else {
                    lon_min
                };
                let lat_rad = if h > 1 {
                    lat_max - (py as f32 / (h - 1) as f32) * (lat_max - lat_min)
                } else {
                    lat_max
                };
                (lon_rad, lat_rad)
            }
        }
    }

    fn shader_coord_mode(&self) -> u32 {
        match self {
            Self::Global => 0,
            Self::Regional { .. } => 1,
        }
    }

    fn is_global(&self) -> bool {
        matches!(self, Self::Global) || self.is_global_extent()
    }

    fn is_global_extent(&self) -> bool {
        let (lon_min, lon_max) = self.lon_bounds_deg();
        let (lat_min, lat_max) = self.lat_bounds_deg();
        (lon_max - lon_min).abs() >= 350.0 && (lat_max - lat_min).abs() >= 160.0
    }

    fn lon_bounds_deg(&self) -> (f32, f32) {
        match self {
            Self::Global => (-180.0, 180.0),
            Self::Regional { lon_bounds, .. } => *lon_bounds,
        }
    }

    fn lat_bounds_deg(&self) -> (f32, f32) {
        match self {
            Self::Global => (-90.0, 90.0),
            Self::Regional { lat_bounds, .. } => *lat_bounds,
        }
    }

    fn requires_geo_coords(&self) -> bool {
        self.is_regional()
    }
}

impl CartesianTopology {
    fn cell_center_from_norm(
        &self,
        norm_x: f32,
        norm_y: f32,
        width: usize,
        height: usize,
    ) -> (usize, usize) {
        let w = width.max(1);
        let h = height.max(1);
        let px = ((norm_x * w as f32).floor() as usize).min(w.saturating_sub(1));
        let py = ((norm_y * h as f32).floor() as usize).min(h.saturating_sub(1));
        (px, py)
    }
}
