//! Integration tests for NetCDF backend.

use crate::data::backends::netcdf::NetCdfBlockStore;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;

#[test]
fn test_create_and_read_netcdf() {
    let _lock = super::test_support::netcdf_lock();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!(
        "octant_test_data_{}_{:?}.nc",
        std::process::id(),
        std::thread::current().id()
    ));
    let path_str = test_file.to_str().unwrap().to_string();

    // Create a test NetCDF file
    {
        let mut file = netcdf::create(&test_file).expect("create netcdf file");
        file.add_dimension("lat", 4).expect("add lat dim");
        file.add_dimension("lon", 5).expect("add lon dim");

        let mut lat_var = file
            .add_variable::<f32>("lat", &["lat"])
            .expect("add lat var");
        lat_var
            .put_values(&[-90.0f32, -30.0, 30.0, 90.0], ..)
            .expect("put lat values");

        let mut lon_var = file
            .add_variable::<f32>("lon", &["lon"])
            .expect("add lon var");
        lon_var
            .put_values(&[-180.0f32, -90.0, 0.0, 90.0, 180.0], ..)
            .expect("put lon values");

        let mut temp_var = file
            .add_variable::<f32>("temperature", &["lat", "lon"])
            .expect("add temperature var");
        temp_var
            .put_attribute("units", "degC")
            .expect("put units attr");
        temp_var
            .put_attribute("long_name", "Surface Temperature")
            .expect("put long_name attr");

        let data: Vec<f32> = (0..20).map(|i| i as f32 * 1.5).collect();
        temp_var.put_values(&data, ..).expect("put temp data");
    }

    // Open with NetCdfBlockStore
    let store = NetCdfBlockStore::open_local(&path_str).expect("open store");
    let vars = store.variables().expect("get variables");
    assert!(vars.contains(&"temperature".to_string()));
    assert!(vars.contains(&"lat".to_string()));
    assert!(vars.contains(&"lon".to_string()));

    // Inspect metadata
    let metadata = store.inspect().expect("inspect metadata");
    assert_eq!(metadata.store_type, "NetCDF");
    let temp_info = metadata
        .variables
        .iter()
        .find(|v| v.name == "temperature")
        .expect("find temperature info");
    assert_eq!(temp_info.shape, vec![4, 5]);
    assert_eq!(temp_info.dimension_names, vec!["lat", "lon"]);
    assert_eq!(temp_info.units.as_deref(), Some("degC"));
    assert_eq!(temp_info.long_name.as_deref(), Some("Surface Temperature"));

    // Fetch full 2D block (Y oriented from North to South for GPU rendering)
    let req = SliceRequest::full_range("temperature", &[4, 5]);
    let block = store.fetch_block(&req).expect("fetch full block");
    assert_eq!(block.shape, vec![4, 5]);
    assert_eq!(block.values.len(), 20);
    // Row 0 of oriented block corresponds to North (+90 lat, row index 3 of raw data: 15..20)
    assert_eq!(block.values[0], 22.5);
    // Row 3 of oriented block corresponds to South (-90 lat, row index 0 of raw data: 0..5)
    assert_eq!(block.values[15], 0.0);

    // Fetch 2D sub-slice
    let sub_req = SliceRequest::new(
        "temperature",
        vec![
            crate::data::slice_request::DimensionSelection::range(1, 3),
            crate::data::slice_request::DimensionSelection::range(2, 5),
        ],
    );
    let sub_block = store.fetch_block(&sub_req).expect("fetch sub block");
    assert_eq!(sub_block.shape, vec![2, 3]);
    assert_eq!(sub_block.values.len(), 6);

    // Clean up
    let _ = std::fs::remove_file(test_file);
}

#[test]
fn test_netcdf_scale_offset_and_fill() {
    let _lock = super::test_support::netcdf_lock();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!(
        "octant_test_scale_fill_{}_{:?}.nc",
        std::process::id(),
        std::thread::current().id()
    ));
    let path_str = test_file.to_str().unwrap().to_string();

    {
        let mut file = netcdf::create(&test_file).expect("create netcdf file");
        file.add_dimension("dim0", 2).expect("add dim0");
        file.add_dimension("dim1", 2).expect("add dim1");

        let mut var = file
            .add_variable::<i16>("calibrated_data", &["dim0", "dim1"])
            .expect("add var");
        var.put_attribute("scale_factor", 0.1f32)
            .expect("scale attr");
        var.put_attribute("add_offset", 10.0f32)
            .expect("offset attr");
        var.put_attribute("_FillValue", -999i16).expect("fill attr");

        // [100, -999, 200, 0] -> expected [100 * 0.1 + 10 = 20.0, NaN, 200 * 0.1 + 10 = 30.0, 0 * 0.1 + 10 = 10.0]
        var.put_values(&[100i16, -999, 200, 0], ..)
            .expect("put values");
    }

    let store = NetCdfBlockStore::open_local(&path_str).expect("open store");
    let req = SliceRequest::full_range("calibrated_data", &[2, 2]);
    let block = store.fetch_block(&req).expect("fetch block");

    assert_eq!(block.values.len(), 4);
    assert!((block.values[0] - 20.0).abs() < 1e-4);
    assert!(block.values[1].is_nan());
    assert!((block.values[2] - 30.0).abs() < 1e-4);
    assert!((block.values[3] - 10.0).abs() < 1e-4);

    let _ = std::fs::remove_file(test_file);
}

#[test]
fn test_netcdf_1d_variable_loading() {
    let _lock = super::test_support::netcdf_lock();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!(
        "octant_test_1d_var_{}_{:?}.nc",
        std::process::id(),
        std::thread::current().id()
    ));
    let path_str = test_file.to_str().unwrap().to_string();

    {
        let mut file = netcdf::create(&test_file).expect("create netcdf file");
        file.add_dimension("lon", 5).expect("add dim");

        let mut lon_var = file
            .add_variable::<f32>("lon", &["lon"])
            .expect("add lon var");
        lon_var
            .put_attribute("units", "degrees_east")
            .expect("units");
        lon_var
            .put_values(&[-180.0f32, -90.0, 0.0, 90.0, 180.0], ..)
            .expect("put values");
    }

    let store = NetCdfBlockStore::open_local(&path_str).expect("open store");
    let req = SliceRequest::full_range("lon", &[5]);
    let block = store.fetch_block(&req).expect("fetch 1d block");

    assert_eq!(block.rank(), 1);
    assert_eq!(block.shape, vec![5]);
    assert_eq!(block.values.len(), 5);
    assert_eq!(block.values[0], -180.0);
    assert_eq!(block.values[4], 180.0);

    // Verify that slicing into 2D MatrixData succeeds with width = 5, height = 1
    let matrix = block
        .slice_2d_with_ranges(0, 0, (0, 5), (0, 1), &[0], 1, "test_1d", true)
        .expect("slice 1d to matrix");
    assert_eq!(matrix.width, 5);
    assert_eq!(matrix.height, 1);
    assert_eq!(matrix.values.len(), 5);
    assert_eq!(matrix.min_val, -180.0);
    assert_eq!(matrix.max_val, 180.0);

    let _ = std::fs::remove_file(test_file);
}
