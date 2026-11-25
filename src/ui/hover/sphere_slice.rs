//! The globe under the slicer and detail tile: which cells the slicer's box
//! keeps, normalized `(u, v)` quad mapping, detail tile relief sampling, and
//! screen-space target projection.

use std::f32::consts::PI;

use egui::Pos2;

use super::camera::Camera3D;
use super::raycast_sphere::SphereField;
use super::smooth::value_at;
use crate::app::OctantApp;
use crate::app::detail::{center_of_edge, edge_of_center};
use crate::data::MatrixData;
use crate::ui::hover::detail::{TileReading, tile_reading};
use crate::utils::height::height_factor;

impl SphereField<'_> {
    /// Whether the slicer keeps `cell`, met toward `(lon, lat)`: by its
    /// center (HEALPix pixels, which have no rows, by where they lie on the
    /// globe) and its value's height, as sphere.wgsl cuts blocks.
    pub(super) fn kept(&self, cell: (usize, usize), lon: f32, lat: f32) -> bool {
        let detail = self.detail_reading(lon, lat);
        let placed = match (self.matrix.grid.is_healpix(), detail.is_some()) {
            (true, _) => self.slice.keeps_uv(lon / (2.0 * PI) + 0.5, 0.5 - lat / PI),
            (false, true) => self
                .uv(lon, lat)
                .is_some_and(|(u, v)| self.slice.keeps_uv(u, v)),
            (false, false) => {
                let grid = (self.matrix.width, self.matrix.height);
                self.slice.keeps_cell(cell, grid)
            }
        };
        let val = detail.unwrap_or_else(|| value_at(self.layer, self.matrix, cell.0, cell.1));
        let h = self.scale.normalized(val);
        placed && self.slice.keeps_height(h, &self.scale, self.relief)
    }

    /// Fractions `(u, v)` of the grid's extent at `(lon, lat)` on a global
    /// or regional regular sphere, with a regional grid's first and last
    /// cell centers at 0 and 1 (`get_lon_lat`), wrapped around its middle.
    pub(super) fn center_uv(&self, lon: f32, lat: f32) -> Option<(f32, f32)> {
        let grid = &self.matrix.grid;
        match grid.sphere_coord_mode() {
            0 => Some(((lon + PI) / (2.0 * PI), 0.5 - lat / PI)),
            1 => {
                let [lon_min, lon_max] = grid.lon_bounds_rad();
                let [lat_min, lat_max] = grid.lat_bounds_rad();
                let lon_mid = 0.5 * (lon_min + lon_max);
                let lon = lon_mid + ((lon - lon_mid + PI).rem_euclid(2.0 * PI) - PI);
                let u = (lon - lon_min) / (lon_max - lon_min).abs().max(1e-6);
                let v = (lat_max - lat) / (lat_max - lat_min).abs().max(1e-6);
                Some((u, v))
            }
            _ => None,
        }
    }

    /// The plotted level's cells along x and y: tiles' `uv` are fractions
    /// of its quad, whatever the drawn mesh's thinning.
    fn plotted_size(&self) -> (usize, usize) {
        let m = self.layer.data.matrix.as_ref().unwrap_or(self.matrix);
        (m.width, m.height)
    }

    /// Quad fractions `(u, v)` of the plotted level at `(lon, lat)`, as
    /// detail tiles' `uv` and the slicer read them.
    pub(super) fn uv(&self, lon: f32, lat: f32) -> Option<(f32, f32)> {
        let (u, v) = self.center_uv(lon, lat)?;
        if self.matrix.grid.sphere_coord_mode() != 1 {
            return Some((u, v));
        }
        let (w, h) = self.plotted_size();
        Some((edge_of_center(u, w), edge_of_center(v, h)))
    }

    /// Longitude and latitude (radians) at quad fractions `(u, v)` (`uv`).
    pub(super) fn lon_lat_of_uv(&self, u: f32, v: f32) -> (f32, f32) {
        let grid = &self.matrix.grid;
        if grid.sphere_coord_mode() == 1 {
            let (w, h) = self.plotted_size();
            let (u, v) = (center_of_edge(u, w), center_of_edge(v, h));
            let [lon_min, lon_max] = grid.lon_bounds_rad();
            let [lat_min, lat_max] = grid.lat_bounds_rad();
            let lon = lon_min + u * (lon_max - lon_min);
            let lat = (lat_max - v * (lat_max - lat_min)).clamp(-PI * 0.5, PI * 0.5);
            (lon, lat)
        } else {
            ((u - 0.5) * 2.0 * PI, (0.5 - v) * PI)
        }
    }

    /// The active detail tile's drawn cell at `(lon, lat)`, when covered.
    pub(super) fn detail_at(&self, lon: f32, lat: f32) -> Option<TileReading> {
        let (u, v) = self.uv(lon, lat).filter(|_| self.tiles)?;
        tile_reading(self.layer, u, v)
    }

    /// Value of the active detail tile at `(lon, lat)`, when covered.
    pub(super) fn detail_reading(&self, lon: f32, lat: f32) -> Option<f32> {
        self.detail_at(lon, lat).map(|r| r.value)
    }

    /// Radii the detail tile's drawn cell at `(lon, lat)` spans, when
    /// covered (`SphereField::span`).
    pub(super) fn detail_span(&self, lon: f32, lat: f32) -> Option<(f32, f32)> {
        let val = self.detail_reading(lon, lat)?;
        let dr = match val.is_finite() {
            true => self.scale.normalized(val) * height_factor(true, self.mode) * self.disp,
            false => 0.0,
        };
        Some(match self.mode {
            3 => (1.0 + dr.min(0.0), 1.0 + dr.max(0.0)),
            _ => (1.0 + dr, 1.0 + dr),
        })
    }

    /// The march's cell identity toward `(lon, lat)`: a detail tile's drawn
    /// cell (past the coarse grid's indices) where one covers it, else
    /// coarse `cell`, so patches and columns follow what is drawn.
    pub(super) fn drawn_identity(
        &self,
        cell: (usize, usize),
        lon: f32,
        lat: f32,
    ) -> (usize, usize) {
        self.detail_at(lon, lat).map_or(cell, |d| {
            (self.matrix.width + d.cell.0, self.matrix.height + d.cell.1)
        })
    }

    /// The march step's angle and its sample budget's factor over the
    /// coarse grid's, when a resident tile's drawn cells (`base`'s
    /// fraction) are finer than the coarse ones (`base`).
    pub(super) fn detail_cell_angle(&self, base: f32) -> (f32, usize) {
        let Some(tile) = (self.layer.data.detail.tile.as_ref()).filter(|_| self.tiles) else {
            return (base, 1);
        };
        let Some(m) = &tile.matrix else {
            return (base, 1);
        };
        let k = (tile.thinning.stride as usize).max(1);
        let (dw, dh) = (m.width.div_ceil(k).max(1), m.height.div_ceil(k).max(1));
        let rx = (tile.uv[2] - tile.uv[0]).max(1e-4) * self.matrix.width.max(1) as f32 / dw as f32;
        let ry = (tile.uv[3] - tile.uv[1]).max(1e-4) * self.matrix.height.max(1) as f32 / dh as f32;
        let ratio = rx.min(ry).min(1.0);
        // shortcut: rays that never reach the tile march at its step too; a
        // tile only exists zoomed in, where it covers most of the view.
        (
            (base * ratio).max(1e-5),
            (1.0 / ratio.max(1e-3)).ceil() as usize,
        )
    }
}

/// Projects spherical coordinates `(lon, lat)` at radius `r` to screen space.
pub(super) fn project_lon_lat(camera: &Camera3D, lon: f32, lat: f32, r: f32) -> Option<Pos2> {
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_lon, cos_lon) = lon.sin_cos();
    camera.project_point([r * cos_lat * sin_lon, r * sin_lat, r * cos_lat * cos_lon])
}

/// Forward projects a hovered detail tile cell's center on the drawn globe to
/// screen space.
pub fn sphere_detail_target_pos(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    camera: &Camera3D,
    cell: crate::ui::hover::detail::DetailCell,
) -> Option<Pos2> {
    let field = SphereField::new(app, matrix?);
    let (lon_c, lat_c) = field.lon_lat_of_uv(cell.center.0, cell.center.1);
    let r = field.detail_span(lon_c, lat_c).map_or(1.0, |s| s.1);
    project_lon_lat(camera, lon_c, lat_c, r)
}

/// Forward projects the hovered cell's center, on the drawn globe, to screen
/// space.
pub fn sphere_target_pos(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    px: usize,
    py: usize,
) -> Option<Pos2> {
    let (lon_c, lat_c) = matrix
        .grid
        .cell_center_lon_lat_rad(px, py, matrix.width, matrix.height);
    let r = SphereField::new(app, matrix).span(px, py, lon_c, lat_c).1;
    project_lon_lat(camera, lon_c, lat_c, r)
}
