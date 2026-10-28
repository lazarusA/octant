//! Where coastlines place the base layer's grid: the outer edges of its cells,
//! as the plots draw them, so coastlines line up with the cells and the axes.

use crate::app::OctantApp;

impl OctantApp {
    /// `(lon_min, lon_max, lat_min, lat_max)` in degrees for the 2D coastlines.
    pub(crate) fn coastline_bounds_2d(&self) -> (f32, f32, f32, f32) {
        if let Some([lon_min, lon_max, lat_min, lat_max]) = self.coastline_cell_edges(false) {
            return (lon_min, lon_max, lat_min, lat_max);
        }
        (self.layers.base.data.matrix.as_ref())
            .map(|m| crate::plots::dataset_geo_bounds(&m.grid))
            .unwrap_or((-180.0, 180.0, 90.0, -90.0))
    }

    /// Longitude and latitude bounds in radians for the surface or sphere coastlines,
    /// else the mesh's own.
    pub(crate) fn coastline_bounds_3d(
        &self,
        sphere: bool,
        mesh: &crate::plots::Mesh3DUniformParams,
    ) -> ([f32; 2], [f32; 2]) {
        match self.coastline_cell_edges(sphere) {
            Some([lon_min, lon_max, lat_min, lat_max]) => (
                [lon_min.to_radians(), lon_max.to_radians()],
                [lat_min.to_radians(), lat_max.to_radians()],
            ),
            None => (mesh.lon_bounds, mesh.lat_bounds),
        }
    }

    /// The base layer's outer cell edges in degrees, `[lon_min, lon_max, lat_min,
    /// lat_max]`, for coastlines over plots that draw whole cells edge to edge: the
    /// heatmap and surface on regular grids, and the sphere on regional ones. `None`
    /// keeps the grid's own bounds: irregular and HEALPix grids, and global grids on
    /// the sphere, which spreads them over the whole globe.
    fn coastline_cell_edges(&self, sphere: bool) -> Option<[f32; 4]> {
        let grid = &self.layers.base.data.matrix.as_ref()?.grid;
        let mode = match sphere {
            true => grid.sphere_coord_mode(),
            false => grid.render_coord_mode(),
        };
        if mode > 1 || (sphere && mode == 0) {
            return None;
        }
        let (lon_min, mut lon_max) = self.plotted_cell_edges(self.get_spatial_dim_index(0))?;
        let (lat_min, lat_max) = self.plotted_cell_edges(self.get_spatial_dim_index(1))?;
        // shortcut: the coastline shaders take lon_max > 180 for a 0..360 grid, so edges
        // of a -180..180 grid just past 180 (MUR SST to 180.005) stop at 180, misplacing
        // the last cell's coastline by its overshoot; teach the shaders the wrap to upgrade.
        // Grids from 0 (edges from -0.125 on ERA5) are 0..360 grids and keep theirs.
        if lon_min < -90.0 && lon_max > 180.0 {
            lon_max = 180.0;
        }
        Some([lon_min, lon_max, lat_min, lat_max].map(|v| v as f32))
    }
}

#[cfg(test)]
mod tests {
    use crate::app::test_support::{memory_app, selection_of};
    use crate::data::{CoordValues, CoordinateGrid, MatrixData};

    /// `elev(lat, lon)` (5x4) on 36° x 90° cells centered from 72°N and -135°E,
    /// plotted on `grid`.
    fn plotted_on(grid: CoordinateGrid) -> crate::app::OctantApp {
        plotted_from(-135.0, grid)
    }

    fn plotted_from(lon_start: f64, grid: CoordinateGrid) -> crate::app::OctantApp {
        let (mut app, mut meta) = memory_app();
        let coords = &mut meta.dimension_coordinates;
        let regular = |start, step, len| CoordValues::Regular { start, step, len };
        coords.insert("lat".into(), regular(72.0, -36.0, 5));
        coords.insert("lon".into(), regular(lon_start, 90.0, 4));
        *app.layers.base.selection_mut() = selection_of(&mut app, &meta, "elev");
        let values = vec![0.0; 20];
        let matrix = MatrixData::new_with_grid(4, 5, values, 0.0, 1.0, "elev".into(), 1, grid);
        app.layers.base.data.matrix = Some(matrix);
        app
    }

    #[test]
    fn coastlines_take_the_cells_outer_edges() {
        let regional = CoordinateGrid::RegionalRegular {
            lon_bounds: (-135.0, 135.0),
            lat_bounds: (-72.0, 72.0),
        };
        let edges = Some([-180.0, 180.0, -90.0, 90.0]);
        let app = plotted_on(regional);
        assert_eq!(app.coastline_cell_edges(false), edges);
        // The sphere draws regional cells around their centers too.
        assert_eq!(app.coastline_cell_edges(true), edges);

        let app = plotted_on(CoordinateGrid::GlobalRegular);
        assert_eq!(app.coastline_cell_edges(false), edges);
        // The sphere spreads global grids over the whole globe: the grid's own bounds.
        assert_eq!(app.coastline_cell_edges(true), None);
    }

    #[test]
    fn edges_past_180_stop_at_180_only_on_minus_180_grids() {
        // Cells from -179° to 181°: the shaders would read 181 as a 0..360 grid.
        let app = plotted_from(-134.0, CoordinateGrid::GlobalRegular);
        assert_eq!(
            app.coastline_cell_edges(false),
            Some([-179.0, 180.0, -90.0, 90.0])
        );
        // Cells from 0° (edges from -45°) are a 0..360 grid: no cap.
        let app = plotted_from(0.0, CoordinateGrid::GlobalRegular);
        assert_eq!(
            app.coastline_cell_edges(false),
            Some([-45.0, 315.0, -90.0, 90.0])
        );
    }
}
