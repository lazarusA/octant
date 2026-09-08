use octant::data::{
    SliceRequest,
    backends::NetCdfBlockStore,
    block_store::BlockStore,
    coordinates::CoordinateGrid,
    procedural::{
        generate_clenshaw_curtis_2d, generate_clenshaw_curtis_coords, generate_gaussian_coords,
        generate_gaussian_grid_2d, generate_stepped_resolution_2d,
        generate_stepped_resolution_coords, generate_stretched_regional_2d,
        generate_stretched_regional_coords,
    },
};
use std::path::PathBuf;

fn get_test_data_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    std::fs::create_dir_all(&dir).ok();
    dir
}

#[test]
fn test_generate_and_load_ground_truth_clenshaw_netcdf() {
    let dir = get_test_data_dir();
    let file_path = dir.join("ground_truth_clenshaw.nc");
    let (nx, ny) = (64, 32);

    {
        let mut file = netcdf::create(&file_path).expect("create clenshaw netcdf file");
        file.add_dimension("lat", ny).expect("add lat dim");
        file.add_dimension("lon", nx).expect("add lon dim");

        let (xs, ys) = generate_clenshaw_curtis_coords(nx, ny);
        let xs_f32: Vec<f32> = xs.iter().map(|&v| v as f32).collect();
        let ys_f32: Vec<f32> = ys.iter().map(|&v| v as f32).collect();

        let mut lat_var = file
            .add_variable::<f32>("lat", &["lat"])
            .expect("add lat var");
        lat_var.put_attribute("units", "degrees_north").ok();
        lat_var.put_values(&ys_f32, ..).expect("put lat values");

        let mut lon_var = file
            .add_variable::<f32>("lon", &["lon"])
            .expect("add lon var");
        lon_var.put_attribute("units", "degrees_east").ok();
        lon_var.put_values(&xs_f32, ..).expect("put lon values");

        let (data, _, _) = generate_clenshaw_curtis_2d(nx, ny, 0);
        let mut var = file
            .add_variable::<f32>("clenshaw_wave", &["lat", "lon"])
            .expect("add clenshaw_wave var");
        var.put_attribute("units", "dimensionless").ok();
        var.put_attribute("long_name", "Clenshaw-Curtis Non-linear Harmonic Wave")
            .ok();
        var.put_values(&data, ..).expect("put data values");
    }

    let store = NetCdfBlockStore::open_local(file_path.to_str().unwrap()).expect("open store");
    let request = SliceRequest::full_range("clenshaw_wave", &[ny, nx]);
    let block = store.fetch_block(&request).expect("fetch block");

    assert_eq!(block.shape, vec![ny, nx]);
    let matrix = block
        .slice_2d(1, 0, &[0, 0], 1, "test", true)
        .expect("slice 2d");
    assert_eq!(matrix.width, nx);
    assert_eq!(matrix.height, ny);

    match &matrix.grid {
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => {
            assert_eq!(coords_x.len(), nx);
            assert_eq!(coords_y.len(), ny);
            assert!((coords_x[0] - (-180.0)).abs() < 1e-3);
            assert!((coords_x[nx - 1] - 180.0).abs() < 1e-3);
            assert!((coords_y[0] - 90.0).abs() < 1e-3);
            assert!((coords_y[ny - 1] - (-90.0)).abs() < 1e-3);
        }
        other => panic!("Expected Irregular1D grid, got {:?}", other),
    }
}

#[test]
fn test_generate_and_load_ground_truth_gaussian_netcdf() {
    let dir = get_test_data_dir();
    let file_path = dir.join("ground_truth_gaussian.nc");
    let (nx, ny) = (64, 32);

    {
        let mut file = netcdf::create(&file_path).expect("create gaussian netcdf file");
        file.add_dimension("lat", ny).expect("add lat dim");
        file.add_dimension("lon", nx).expect("add lon dim");

        let (xs, ys) = generate_gaussian_coords(nx, ny);
        let xs_f32: Vec<f32> = xs.iter().map(|&v| v as f32).collect();
        let ys_f32: Vec<f32> = ys.iter().map(|&v| v as f32).collect();

        let mut lat_var = file
            .add_variable::<f32>("lat", &["lat"])
            .expect("add lat var");
        lat_var.put_attribute("units", "degrees_north").ok();
        lat_var.put_values(&ys_f32, ..).expect("put lat values");

        let mut lon_var = file
            .add_variable::<f32>("lon", &["lon"])
            .expect("add lon var");
        lon_var.put_attribute("units", "degrees_east").ok();
        lon_var.put_values(&xs_f32, ..).expect("put lon values");

        let (data, _, _) = generate_gaussian_grid_2d(nx, ny, 0);
        let mut var = file
            .add_variable::<f32>("gaussian_wave", &["lat", "lon"])
            .expect("add gaussian_wave var");
        var.put_attribute("units", "K").ok();
        var.put_attribute("long_name", "Gaussian Latitude Rossby Wave")
            .ok();
        var.put_values(&data, ..).expect("put data values");
    }

    let store = NetCdfBlockStore::open_local(file_path.to_str().unwrap()).expect("open store");
    let request = SliceRequest::full_range("gaussian_wave", &[ny, nx]);
    let block = store.fetch_block(&request).expect("fetch block");

    let matrix = block
        .slice_2d(1, 0, &[0, 0], 1, "test", true)
        .expect("slice 2d");
    assert_eq!(matrix.width, nx);
    assert_eq!(matrix.height, ny);

    match &matrix.grid {
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => {
            assert_eq!(coords_x.len(), nx);
            assert_eq!(coords_y.len(), ny);
            assert!((coords_x[0] - (-180.0)).abs() < 1e-3);
            assert!(coords_y[0] > 0.0 && coords_y[ny - 1] < 0.0);
        }
        other => panic!(
            "Expected Irregular1D grid for Gaussian latitudes, got {:?}",
            other
        ),
    }
}

#[test]
fn test_generate_and_load_ground_truth_stretched_regional_netcdf() {
    let dir = get_test_data_dir();
    let file_path = dir.join("ground_truth_stretched.nc");
    let (nx, ny) = (48, 32);

    {
        let mut file = netcdf::create(&file_path).expect("create stretched netcdf file");
        file.add_dimension("lat", ny).expect("add lat dim");
        file.add_dimension("lon", nx).expect("add lon dim");

        let (xs, ys) = generate_stretched_regional_coords(nx, ny);
        let xs_f32: Vec<f32> = xs.iter().map(|&v| v as f32).collect();
        let ys_f32: Vec<f32> = ys.iter().map(|&v| v as f32).collect();

        let mut lat_var = file
            .add_variable::<f32>("lat", &["lat"])
            .expect("add lat var");
        lat_var.put_attribute("units", "degrees_north").ok();
        lat_var.put_values(&ys_f32, ..).expect("put lat values");

        let mut lon_var = file
            .add_variable::<f32>("lon", &["lon"])
            .expect("add lon var");
        lon_var.put_attribute("units", "degrees_east").ok();
        lon_var.put_values(&xs_f32, ..).expect("put lon values");

        let (data, _, _) = generate_stretched_regional_2d(nx, ny);
        let mut var = file
            .add_variable::<f32>("stretched_checkerboard", &["lat", "lon"])
            .expect("add stretched_checkerboard var");
        var.put_attribute("units", "dimensionless").ok();
        var.put_attribute("long_name", "Geometrically Stretched Regional Checkerboard")
            .ok();
        var.put_values(&data, ..).expect("put data values");
    }

    let store = NetCdfBlockStore::open_local(file_path.to_str().unwrap()).expect("open store");
    let request = SliceRequest::full_range("stretched_checkerboard", &[ny, nx]);
    let block = store.fetch_block(&request).expect("fetch block");

    let matrix = block
        .slice_2d(1, 0, &[0, 0], 1, "test", true)
        .expect("slice 2d");
    assert_eq!(matrix.width, nx);
    assert_eq!(matrix.height, ny);

    match &matrix.grid {
        CoordinateGrid::Irregular1D {
            coords_x,
            coords_y,
            lon_bounds,
            lat_bounds,
        } => {
            assert_eq!(coords_x.len(), nx);
            assert_eq!(coords_y.len(), ny);
            assert!((lon_bounds.0 - 10.0).abs() < 1e-3);
            assert!((lon_bounds.1 - 50.0).abs() < 1e-3);
            assert!((lat_bounds.0 - 30.0).abs() < 1e-3);
            assert!((lat_bounds.1 - 60.0).abs() < 1e-3);
        }
        other => panic!(
            "Expected Irregular1D grid for stretched regional, got {:?}",
            other
        ),
    }
}

#[test]
fn test_generate_and_load_ground_truth_stepped_netcdf() {
    let dir = get_test_data_dir();
    let file_path = dir.join("ground_truth_stepped.nc");
    let (nx, ny) = (64, 32);

    {
        let mut file = netcdf::create(&file_path).expect("create stepped netcdf file");
        file.add_dimension("lat", ny).expect("add lat dim");
        file.add_dimension("lon", nx).expect("add lon dim");

        let (xs, ys) = generate_stepped_resolution_coords(nx, ny);
        let xs_f32: Vec<f32> = xs.iter().map(|&v| v as f32).collect();
        let ys_f32: Vec<f32> = ys.iter().map(|&v| v as f32).collect();

        let mut lat_var = file
            .add_variable::<f32>("lat", &["lat"])
            .expect("add lat var");
        lat_var.put_attribute("units", "degrees_north").ok();
        lat_var.put_values(&ys_f32, ..).expect("put lat values");

        let mut lon_var = file
            .add_variable::<f32>("lon", &["lon"])
            .expect("add lon var");
        lon_var.put_attribute("units", "degrees_east").ok();
        lon_var.put_values(&xs_f32, ..).expect("put lon values");

        let (data, _, _) = generate_stepped_resolution_2d(nx, ny);
        let mut var = file
            .add_variable::<f32>("stepped_stripes", &["lat", "lon"])
            .expect("add stepped_stripes var");
        var.put_attribute("units", "dimensionless").ok();
        var.put_attribute("long_name", "Stepped Multi-Resolution Resolution Jump")
            .ok();
        var.put_values(&data, ..).expect("put data values");
    }

    let store = NetCdfBlockStore::open_local(file_path.to_str().unwrap()).expect("open store");
    let request = SliceRequest::full_range("stepped_stripes", &[ny, nx]);
    let block = store.fetch_block(&request).expect("fetch block");

    let matrix = block
        .slice_2d(1, 0, &[0, 0], 1, "test", true)
        .expect("slice 2d");
    assert_eq!(matrix.width, nx);
    assert_eq!(matrix.height, ny);

    match &matrix.grid {
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => {
            assert_eq!(coords_x.len(), nx);
            assert_eq!(coords_y.len(), ny);
            assert!((coords_x[0] - (-40.0)).abs() < 1e-3);
            assert!((coords_x[nx - 1] - 40.0).abs() < 1e-3);
        }
        other => panic!(
            "Expected Irregular1D grid for stepped resolution, got {:?}",
            other
        ),
    }
}
