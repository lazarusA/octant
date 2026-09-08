//! Geographic and Cartesian coordinate mapping system.

pub mod detection;
pub mod lut;
pub mod search;
pub mod types;

pub use detection::{detect_grid, normalize_grid};
pub use lut::{build_1d_coord_lut, compute_coord_lut_size};
pub use search::find_coord_cell_1d;
pub use types::{CoordinateGrid, GridGeometry, GridKind};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_find_coord_cell_1d_ascending() {
        let coords = [10.0, 20.0, 40.0, 80.0];
        assert_eq!(find_coord_cell_1d(&coords, 5.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 10.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 14.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 16.0), 1);
        assert_eq!(find_coord_cell_1d(&coords, 35.0), 2);
        assert_eq!(find_coord_cell_1d(&coords, 70.0), 3);
        assert_eq!(find_coord_cell_1d(&coords, 100.0), 3);
    }

    #[test]
    fn test_find_coord_cell_1d_descending() {
        let coords = [80.0, 40.0, 20.0, 10.0];
        assert_eq!(find_coord_cell_1d(&coords, 100.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 70.0), 0);
        assert_eq!(find_coord_cell_1d(&coords, 35.0), 1);
        assert_eq!(find_coord_cell_1d(&coords, 16.0), 2);
        assert_eq!(find_coord_cell_1d(&coords, 5.0), 3);
    }

    #[test]
    fn test_cell_center_norm_irregular() {
        let coords_x: Arc<[f32]> = Arc::new([0.0, 10.0, 30.0, 100.0]);
        let coords_y: Arc<[f32]> = Arc::new([0.0, 50.0, 100.0]);
        let grid = CoordinateGrid::Irregular1D {
            coords_x,
            coords_y,
            lon_bounds: (0.0, 100.0),
            lat_bounds: (0.0, 100.0),
        };

        let (u_c, v_c) = grid.cell_center_norm(1, 1, 4, 3);
        assert!((u_c - 0.1).abs() < 1e-5);
        assert!((v_c - 0.5).abs() < 1e-5);

        let (px, py) = grid.find_cell_from_norm(0.08, 0.5, 4, 3);
        assert_eq!(px, 1);
        assert_eq!(py, 1);
    }

    #[test]
    fn same_geometry_detects_coordinate_changes() {
        let first = CoordinateGrid::Irregular1D {
            coords_x: Arc::from([0.0, 10.0, 30.0]),
            coords_y: Arc::from([0.0, 50.0, 100.0]),
            lon_bounds: (0.0, 30.0),
            lat_bounds: (0.0, 100.0),
        };
        let same = first.clone();
        let changed = CoordinateGrid::Irregular1D {
            coords_x: Arc::from([0.0, 11.0, 30.0]),
            coords_y: Arc::from([0.0, 50.0, 100.0]),
            lon_bounds: (0.0, 30.0),
            lat_bounds: (0.0, 100.0),
        };

        assert!(first.same_geometry(&same));
        assert!(!first.same_geometry(&changed));
    }

    #[test]
    fn detects_regular_global_grid() {
        let lons: Vec<f64> = (0..360).map(|i| -180.0 + i as f64).collect();
        let lats: Vec<f64> = (0..181).map(|i| -90.0 + i as f64).collect();

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 360, 181);
        assert_eq!(grid, CoordinateGrid::GlobalRegular);
        assert_eq!(grid.coord_mode(), 0);
    }

    #[test]
    fn detects_regular_regional_grid() {
        let lons: Vec<f64> = (0..50).map(|i| 10.0 + i as f64 * 0.5).collect();
        let lats: Vec<f64> = (0..40).map(|i| 35.0 + i as f64 * 0.5).collect();

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 50, 40);
        match grid {
            CoordinateGrid::RegionalRegular {
                lon_bounds,
                lat_bounds,
            } => {
                assert!((lon_bounds.0 - 10.0).abs() < 1e-3);
                assert!((lat_bounds.0 - 35.0).abs() < 1e-3);
            }
            other => panic!("Expected RegionalRegular, got {:?}", other),
        }
        assert_eq!(grid.coord_mode(), 1);
    }

    #[test]
    fn detects_irregular_1d_grid() {
        let lons: Vec<f64> = (0..50).map(|i| 10.0 + i as f64 * 0.5).collect();
        let mut lats = Vec::new();
        let mut curr = 30.0;
        for i in 0..40 {
            lats.push(curr);
            curr += 0.5 + (i as f64 * 0.05);
        }

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 50, 40);
        assert_eq!(grid.coord_mode(), 2);
        assert!(matches!(grid, CoordinateGrid::Irregular1D { .. }));
        assert!(!grid.is_global());
    }

    #[test]
    fn detects_global_irregular_1d_grid() {
        let lons: Vec<f64> = (0..360).map(|i| -180.0 + i as f64).collect();
        let mut lats = Vec::new();
        let mut curr = -90.0;
        for _ in 0..180 {
            lats.push(curr);
            let lat_rad = (curr as f32).to_radians();
            let step = 1.0 + (lat_rad.cos() as f64) * 0.5;
            curr += step;
        }

        let grid = CoordinateGrid::detect_grid("lon", "lat", Some(&lons), Some(&lats), 360, 180);
        assert_eq!(grid.coord_mode(), 2);
        assert!(matches!(grid, CoordinateGrid::Irregular1D { .. }));
        assert!(grid.is_global());
    }

    #[test]
    fn normalize_grid_exposes_geometry_contract() {
        let lons: Vec<f64> = (0..360).map(|i| -180.0 + i as f64).collect();
        let lats: Vec<f64> = (0..181).map(|i| -90.0 + i as f64).collect();

        let geometry = normalize_grid("lon", "lat", Some(&lons), Some(&lats), 360, 181);
        assert_eq!(geometry.kind, GridKind::GlobalRegular);
        assert!(geometry.is_global_extent());
        assert!(!geometry.requires_geo_coords());
    }

    #[test]
    fn test_coord_lut_matches_binary_search() {
        let w = 256;
        let coords: Vec<f32> = (0..w)
            .map(|i| {
                let t = i as f32 / (w - 1) as f32;
                t * t * 100.0
            })
            .collect();

        let lut_size = compute_coord_lut_size(w);
        assert_eq!(lut_size, 4096);
        let lut = build_1d_coord_lut(&coords, lut_size);
        assert_eq!(lut.len(), 4096);

        for sample_k in 0..1000 {
            let u = sample_k as f32 / 999.0;
            let target = coords[0] + u * (coords[coords.len() - 1] - coords[0]);
            let exact_cell = find_coord_cell_1d(&coords, target);

            let lut_idx = ((u * (lut_size - 1) as f32) + 0.5) as usize;
            let lut_cell = lut[lut_idx.min(lut_size - 1)] as usize;

            let diff = (exact_cell as isize - lut_cell as isize).abs();
            assert!(
                diff <= 1,
                "LUT cell {lut_cell} deviated from exact cell {exact_cell} at u={u}"
            );
        }
    }

    #[test]
    fn test_coord_lut_descending() {
        let coords = [100.0, 75.0, 30.0, 10.0, 0.0];
        let lut = build_1d_coord_lut(&coords, 4096);
        assert_eq!(lut[0], 0.0);
        assert_eq!(lut[4095], 4.0);
    }
}
