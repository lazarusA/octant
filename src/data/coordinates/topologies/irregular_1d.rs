//! 1D non-uniform rectilinear coordinate grid topology.

use crate::data::coordinates::search::find_coord_cell_1d;
use crate::data::coordinates::topology::GridTopology;
use crate::plots::PlotType;
use std::sync::Arc;

static IRREGULAR_PLOT_TYPES: &[PlotType] = &[
    PlotType::Heatmap,
    PlotType::Line,
    PlotType::Surface,
    PlotType::Volume,
    PlotType::Sphere,
    PlotType::PointCloud,
];

/// 1D non-uniform coordinate arrays along X and Y axes (e.g. Gaussian latitude grids, stretched grids).
#[derive(Clone, Debug, PartialEq)]
pub struct Irregular1DTopology {
    pub coords_x: Arc<[f32]>,
    pub coords_y: Arc<[f32]>,
    pub lon_bounds: (f32, f32),
    pub lat_bounds: (f32, f32),
}

impl Irregular1DTopology {
    #[inline]
    pub fn default_supported_plots() -> &'static [PlotType] {
        IRREGULAR_PLOT_TYPES
    }

    pub fn cell_from_norm_slices(
        cx: &[f32],
        cy: &[f32],
        norm_x: f32,
        norm_y: f32,
        w: usize,
        h: usize,
    ) -> (usize, usize) {
        let (w, h) = (w.max(1), h.max(1));
        let px = if cx.len() >= 2 {
            find_coord_cell_1d(cx, cx[0] + norm_x * (cx[cx.len() - 1] - cx[0]))
        } else {
            ((norm_x * w as f32).floor() as usize).min(w.saturating_sub(1))
        };
        let py = if cy.len() >= 2 {
            find_coord_cell_1d(cy, cy[0] + norm_y * (cy[cy.len() - 1] - cy[0]))
        } else {
            ((norm_y * h as f32).floor() as usize).min(h.saturating_sub(1))
        };
        (px.min(w.saturating_sub(1)), py.min(h.saturating_sub(1)))
    }

    pub fn lon_lat_to_cell_slices(
        cx: &[f32],
        cy: &[f32],
        lon_rad: f32,
        lat_rad: f32,
        w: usize,
        h: usize,
    ) -> Option<(usize, usize)> {
        let (w, h) = (w.max(1), h.max(1));
        let mut lon_deg = lon_rad.to_degrees();
        if let (Some(&fx), Some(&lx)) = (cx.first(), cx.last())
            && fx.min(lx) >= -5.0
            && fx.max(lx) > 180.0
            && lon_deg < 0.0
        {
            lon_deg += 360.0;
        }

        let px = if cx.len() >= 2 {
            find_coord_cell_1d(cx, lon_deg)
        } else {
            let u =
                ((lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)).clamp(0.0, 1.0);
            ((u * w as f32).floor() as usize).min(w.saturating_sub(1))
        };
        let py = if cy.len() >= 2 {
            find_coord_cell_1d(cy, lat_rad.to_degrees())
        } else {
            let v = (0.5 - (lat_rad / std::f32::consts::PI)).clamp(0.0, 1.0);
            ((v * h as f32).floor() as usize).min(h.saturating_sub(1))
        };
        Some((px.min(w.saturating_sub(1)), py.min(h.saturating_sub(1))))
    }

    pub fn cell_center_norm_slices(
        cx: &[f32],
        cy: &[f32],
        px: usize,
        py: usize,
        w: usize,
        h: usize,
    ) -> (f32, f32) {
        let u_c = if cx.len() >= 2 {
            let (fx, lx) = (cx[0], cx[cx.len() - 1]);
            let span = if (lx - fx).abs() > 1e-6 { lx - fx } else { 1.0 };
            ((cx.get(px).copied().unwrap_or(fx) - fx) / span).clamp(0.0, 1.0)
        } else {
            (px as f32 + 0.5) / w.max(1) as f32
        };
        let v_c = if cy.len() >= 2 {
            let (fy, ly) = (cy[0], cy[cy.len() - 1]);
            let span = if (ly - fy).abs() > 1e-6 { ly - fy } else { 1.0 };
            ((cy.get(py).copied().unwrap_or(fy) - fy) / span).clamp(0.0, 1.0)
        } else {
            (py as f32 + 0.5) / h.max(1) as f32
        };
        (u_c, v_c)
    }

    #[inline]
    pub fn cell_center_lon_lat_rad_slices(
        cx: &[f32],
        cy: &[f32],
        px: usize,
        py: usize,
    ) -> (f32, f32) {
        let deg_lon = cx.get(px).copied().unwrap_or(0.0);
        let deg_lat = cy.get(py).copied().unwrap_or(0.0);
        (deg_lon.to_radians(), deg_lat.to_radians())
    }
}

impl GridTopology for Irregular1DTopology {
    fn name(&self) -> &'static str {
        "Rectilinear 1D (Irregular)"
    }
    fn spatial_rank(&self) -> usize {
        2
    }
    fn supported_plot_types(&self) -> &'static [PlotType] {
        IRREGULAR_PLOT_TYPES
    }

    fn data_aspect_ratio(&self, width: usize, height: usize) -> f32 {
        if self.is_global() {
            2.0
        } else {
            (width.max(1) as f32 / height.max(1) as f32).clamp(0.1, 10.0)
        }
    }

    fn surface_uv_to_cell(&self, u: f32, v: f32, width: usize, height: usize) -> (usize, usize) {
        Self::cell_from_norm_slices(
            &self.coords_x,
            &self.coords_y,
            u.clamp(0.0, 1.0),
            v.clamp(0.0, 1.0),
            width,
            height,
        )
    }

    fn norm_to_cell(
        &self,
        norm_x: f32,
        norm_y: f32,
        width: usize,
        height: usize,
    ) -> (usize, usize) {
        Self::cell_from_norm_slices(
            &self.coords_x,
            &self.coords_y,
            norm_x.clamp(0.0, 1.0),
            norm_y.clamp(0.0, 1.0),
            width,
            height,
        )
    }

    fn lon_lat_to_cell(
        &self,
        lon_rad: f32,
        lat_rad: f32,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        Self::lon_lat_to_cell_slices(
            &self.coords_x,
            &self.coords_y,
            lon_rad,
            lat_rad,
            width,
            height,
        )
    }

    fn cell_center_norm(&self, px: usize, py: usize, width: usize, height: usize) -> (f32, f32) {
        Self::cell_center_norm_slices(&self.coords_x, &self.coords_y, px, py, width, height)
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

    fn cell_center_lon_lat_rad(&self, px: usize, py: usize, _w: usize, _h: usize) -> (f32, f32) {
        Self::cell_center_lon_lat_rad_slices(&self.coords_x, &self.coords_y, px, py)
    }

    fn shader_coord_mode(&self) -> u32 {
        2
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
