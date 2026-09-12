use octant::data::backends::procedural::ProceduralBlockStore;
use octant::data::block_store::BlockStore;
use octant::data::coordinates::healpix::*;
use octant::data::slice_request::{DimensionSelection, SliceRequest};

#[test]
fn test_healpix_speedyweather_nside16_first_ring() {
    let nside = 16;
    let npix = nside_to_npix(nside);
    assert_eq!(npix, 3072);

    // Exact values from SpeedyWeather.jl / cuHPX:
    // lat = 87.07581964294992
    // lon = [45.0, 135.0, 225.0, 315.0]
    // ring = [1, 1, 1, 1]
    let expected_lat_deg = 87.07582;
    let expected_lons_deg = [45.0f32, 135.0, 225.0, 315.0];

    for (i, &expected_lon_deg) in expected_lons_deg.iter().enumerate() {
        let (lon_rad, lat_rad) = pix2ang_ring(nside, i);
        let lon_deg = lon_rad.to_degrees();
        let lat_deg = lat_rad.to_degrees();
        let (ring, in_ring) = pix2ring(nside, i);

        assert_eq!(ring, 1, "Pixel {i} must be on ring 1");
        assert_eq!(in_ring, i, "Pixel {i} must have index {i} in ring");
        assert!(
            (lat_deg - expected_lat_deg).abs() < 1e-3,
            "Lat mismatch: {lat_deg} vs {expected_lat_deg}"
        );
        assert!(
            (lon_deg - expected_lon_deg).abs() < 1e-3,
            "Lon mismatch: {lon_deg} vs {expected_lon_deg}"
        );

        let recovered_p = ang2pix_ring(nside, lon_rad, lat_rad);
        assert_eq!(recovered_p, i, "Round-trip pixel mismatch at {i}");
    }
}

#[test]
fn test_healpix_round_trip_resolutions() {
    for &nside in &[1, 2, 4, 8, 16] {
        let npix = nside_to_npix(nside);
        for p in 0..npix {
            let (lon_rad, lat_rad) = pix2ang_ring(nside, p);
            let recovered_p = ang2pix_ring(nside, lon_rad, lat_rad);
            assert_eq!(
                recovered_p, p,
                "Failed round-trip at nside={nside}, pixel={p}"
            );
        }
    }
}

#[test]
fn test_healpix_procedural_store_inspect_and_slice() {
    let store =
        ProceduralBlockStore::open("procedural://healpix").expect("open procedural healpix");
    let meta = store.inspect().expect("inspect healpix store");

    assert_eq!(meta.variables.len(), 5);
    let temp_var = meta
        .variables
        .iter()
        .find(|v| v.name == "temp")
        .expect("temp variable");
    assert_eq!(temp_var.dimension_names, vec!["time", "layer", "cell"]);
    assert_eq!(temp_var.shape, vec![12, 8, 3072]);

    let mslp_var = meta
        .variables
        .iter()
        .find(|v| v.name == "mslp")
        .expect("mslp variable");
    assert_eq!(mslp_var.dimension_names, vec!["time", "cell"]);
    assert_eq!(mslp_var.shape, vec![12, 3072]);

    // Fetch a 2D slice of temp[t=0, layer=0, cell=0..3072]
    let request = SliceRequest {
        variable: "temp".to_string(),
        selections: vec![
            DimensionSelection::Index(0),
            DimensionSelection::Index(0),
            DimensionSelection::Range {
                start: 0,
                end: 3072,
            },
        ],
    };

    let block = store.fetch_block(&request).expect("fetch temp block");
    assert_eq!(block.shape, vec![3072]);
    assert_eq!(block.values.len(), 3072);

    let matrix = block
        .slice_2d(0, 0, &[0], 12, "temp", true)
        .expect("slice 2d matrix");
    assert_eq!(matrix.width, 3072);
    assert_eq!(matrix.height, 1);
    assert!(matrix.grid.is_healpix());
    assert!(matrix.grid.is_global());

    // Verify cell query at North Pole (lat = 90) maps to first ring pixel
    let cell_idx =
        matrix
            .grid
            .find_cell_from_lon_lat_rad(std::f32::consts::FRAC_PI_4, 1.519759, 3072, 1);
    assert_eq!(cell_idx, Some((0, 0)));

    let (lon_rad, lat_rad) = matrix.grid.cell_center_lon_lat_rad(0, 0, 3072, 1);
    assert!((lat_rad.to_degrees() - 87.07582).abs() < 1e-3);
    assert!((lon_rad.to_degrees() - 45.0).abs() < 1e-3);
}

#[test]
fn test_healpix_cell_boundaries() {
    let nside = 16;
    let bounds = pix_boundaries(nside, 0);
    assert_eq!(bounds.len(), 4);
    // North, East, South, West vertices are non-zero and bounded
    for (lon, lat) in bounds {
        assert!((0.0..=2.0 * std::f32::consts::PI).contains(&lon));
        assert!((-std::f32::consts::FRAC_PI_2..=std::f32::consts::FRAC_PI_2).contains(&lat));
    }
}

#[test]
fn test_healpix_slider_auto_init_and_axes() {
    use octant::app::{AnimationRole, OctantApp, SpatialRole};
    use octant::ui::variables_panel::init_variable_dimension_defaults;

    let store =
        ProceduralBlockStore::open("procedural://healpix").expect("open procedural healpix");
    let meta = store.inspect().expect("inspect healpix store");

    let temp_var = meta
        .variables
        .iter()
        .find(|v| v.name == "temp")
        .expect("temp variable");

    let mut app = OctantApp::default();
    init_variable_dimension_defaults(&mut app, temp_var);

    // Dim 0 = "time" -> Animated
    // Dim 1 = "layer" -> Z
    // Dim 2 = "cell" -> Grid
    assert_eq!(app.dim_config[0].animation, AnimationRole::Animated);
    assert_eq!(app.dim_config[1].spatial, SpatialRole::Z);
    assert_eq!(app.dim_config[2].spatial, SpatialRole::Grid);
    assert_eq!(app.active_plot_type, octant::plots::PlotType::Heatmap);

    let (x_dim, y_dim, z_dim) = OctantApp::resolve_spatial_axes(
        temp_var.shape.len(),
        &temp_var.dimension_names,
        &temp_var.dimension_names,
        &app.dim_config,
    );
    assert_eq!(x_dim, 2);
    assert_eq!(y_dim, 2);
    assert_eq!(z_dim, 1);
}

#[test]
fn test_healpix_cell_index_to_lon_lat_mapping() {
    use octant::data::coordinates::types::CoordinateGrid;
    let nside = 16;
    let npix = nside_to_npix(nside);
    let grid = CoordinateGrid::Healpix {
        nside,
        ordering: HealpixOrder::Ring,
        npix,
        coords_lon: None,
        coords_lat: None,
    };

    // Test a sample of cell indices from north pole, equator, and south pole
    let test_cells = [0, 100, 500, 1536, 2500, 3071];
    for &cell in &test_cells {
        // 1. Map cell index -> (lon, lat)
        let (lon_rad, lat_rad) = grid.cell_center_lon_lat_rad(cell, 0, npix, 1);
        let lon_deg = lon_rad.to_degrees();
        let lat_deg = lat_rad.to_degrees();

        assert!((-90.0..=90.0).contains(&lat_deg));
        assert!((0.0..=360.0).contains(&lon_deg));

        // 2. Map (lon, lat) -> cell index
        let found = grid.find_cell_from_lon_lat_rad(lon_rad, lat_rad, npix, 1);
        assert_eq!(
            found,
            Some((cell, 0)),
            "Cell {cell} at (lon={lon_deg}, lat={lat_deg}) failed round-trip"
        );

        // 3. Map cell index -> normalized viewport (u, v) -> cell index
        let (u_c, v_c) = grid.cell_center_norm(cell, 0, npix, 1);
        assert!((0.0..=1.0).contains(&u_c));
        assert!((0.0..=1.0).contains(&v_c));

        let (recovered_cell, _) = grid.find_cell_from_norm(u_c, v_c, npix, 1);
        assert_eq!(
            recovered_cell, cell,
            "Cell {cell} normalized coordinate round-trip failed at (u={u_c}, v={v_c})"
        );
    }
}

#[test]
fn test_healpix_nested_eerie_remote_compatibility() {
    use octant::data::coordinates::types::CoordinateGrid;
    use octant::data::octant_block::OctantBlock;
    use std::collections::HashMap;

    // Matches the user's remote EERIE climate dataset:
    // Shape: [1, 196608], zoom: 7 => nside = 2^7 = 128, npix = 12 * 128^2 = 196608
    let nside = 128;
    let npix = nside_to_npix(nside);
    assert_eq!(npix, 196608);

    let mut attributes = HashMap::new();
    attributes.insert("healpix_nest".to_string(), "true".to_string());
    attributes.insert("healpix_zoom".to_string(), "7".to_string());
    attributes.insert("standard_name".to_string(), "pr".to_string());

    let values: Vec<f32> = vec![0.0012; npix];
    let block = OctantBlock::new(
        "pr".to_string(),
        vec![npix],
        vec!["cell".to_string()],
        vec![0],
        values,
        HashMap::new(),
        attributes,
    );

    let matrix = block
        .slice_2d(0, 0, &[0], 1, "pr", true)
        .expect("slice 2d matrix");

    assert_eq!(matrix.width, 196608);
    assert_eq!(matrix.height, 1);
    match &matrix.grid {
        CoordinateGrid::Healpix {
            nside: actual_nside,
            ordering,
            npix: actual_npix,
            ..
        } => {
            assert_eq!(*actual_nside, 128);
            assert_eq!(*actual_npix, 196608);
            assert_eq!(*ordering, HealpixOrder::Nested);
            assert_eq!(matrix.grid.render_coord_mode(), 5);
        }
        other => panic!("Expected Nested Healpix grid, got {:?}", other),
    }

    // Test coordinate round-trip on nested scheme
    let test_cells = [0, 100, 1000, 50000, 100000, 196607];
    for &cell in &test_cells {
        let (lon_rad, lat_rad) = matrix.grid.cell_center_lon_lat_rad(cell, 0, npix, 1);
        let found = matrix
            .grid
            .find_cell_from_lon_lat_rad(lon_rad, lat_rad, npix, 1);
        assert_eq!(
            found,
            Some((cell, 0)),
            "Nested cell {cell} failed lon/lat round-trip"
        );
    }
}
