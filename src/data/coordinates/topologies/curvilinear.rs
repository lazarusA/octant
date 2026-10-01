//! 2D curvilinear coordinate grid topology.

use crate::data::coordinates::topology::GridTopology;
use crate::plots::PlotType;
use std::sync::Arc;

static CURVILINEAR_PLOT_TYPES: &[PlotType] = &[
    PlotType::Heatmap,
    PlotType::Line,
    PlotType::Surface,
    PlotType::Volume,
    PlotType::Sphere,
    PlotType::PointCloud,
];

/// 2D curvilinear coordinates (lon(y,x), lat(y,x)) of shape (height * width).
#[derive(Clone, Debug, PartialEq)]
pub struct CurvilinearTopology {
    pub lons: Arc<[f32]>,
    pub lats: Arc<[f32]>,
    pub lon_bounds: (f32, f32),
    pub lat_bounds: (f32, f32),
}

impl CurvilinearTopology {
    #[inline]
    pub fn default_supported_plots() -> &'static [PlotType] {
        CURVILINEAR_PLOT_TYPES
    }

    pub fn cell_center_lon_lat_rad_slices(
        lons: &[f32],
        lats: &[f32],
        px: usize,
        py: usize,
        width: usize,
        height: usize,
    ) -> (f32, f32) {
        let idx = py * width + px;
        if let (Some(&lon_deg), Some(&lat_deg)) = (lons.get(idx), lats.get(idx)) {
            (lon_deg.to_radians(), lat_deg.to_radians())
        } else {
            let w = width.max(1);
            let h = height.max(1);
            let u_c = (px as f32 + 0.5) / w as f32;
            let v_c = (py as f32 + 0.5) / h as f32;
            let lon_rad = (u_c - 0.5) * 2.0 * std::f32::consts::PI;
            let lat_rad = (0.5 - v_c) * std::f32::consts::PI;
            (lon_rad, lat_rad)
        }
    }
}

impl GridTopology for CurvilinearTopology {
    fn name(&self) -> &'static str {
        "Curvilinear 2D"
    }

    fn spatial_rank(&self) -> usize {
        2
    }

    fn supported_plot_types(&self) -> &'static [PlotType] {
        CURVILINEAR_PLOT_TYPES
    }

    fn data_aspect_ratio(&self, width: usize, height: usize) -> f32 {
        if self.is_global() {
            2.0
        } else {
            let w = width.max(1) as f32;
            let h = height.max(1) as f32;
            (w / h).clamp(0.1, 10.0)
        }
    }

    fn surface_uv_to_cell(&self, u: f32, v: f32, width: usize, height: usize) -> (usize, usize) {
        let w = width.max(1);
        let h = height.max(1);
        let px = ((u.clamp(0.0, 1.0) * w as f32).floor() as usize).min(w.saturating_sub(1));
        let py = ((v.clamp(0.0, 1.0) * h as f32).floor() as usize).min(h.saturating_sub(1));
        (px, py)
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
        let u = ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)).clamp(0.0, 1.0);
        let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
        let px = ((u * w as f32).floor() as usize).min(w.saturating_sub(1));
        let py = ((v * h as f32).floor() as usize).min(h.saturating_sub(1));
        Some((px, py))
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
        Self::cell_center_lon_lat_rad_slices(&self.lons, &self.lats, px, py, width, height)
    }

    fn shader_coord_mode(&self) -> u32 {
        3
    }

    fn is_global(&self) -> bool {
        self.is_global_extent()
    }

    fn is_global_extent(&self) -> bool {
        let (lon_min, lon_max) = self.lon_bounds_deg();
        let (lat_min, lat_max) = self.lat_bounds_deg();
        (lon_max - lon_min).abs() >= 350.0 && (lat_max - lat_min).abs() >= 160.0
    }

    fn lon_bounds_deg(&self) -> (f32, f32) {
        self.lon_bounds
    }

    fn lat_bounds_deg(&self) -> (f32, f32) {
        self.lat_bounds
    }
}
