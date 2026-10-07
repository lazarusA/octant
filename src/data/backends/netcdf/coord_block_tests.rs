//! NetCDF blocks end to end: coordinates of uneven flipped rows and of grouped variables.

use super::test_support::{TempNc, inspect, netcdf_lock, put};

/// A 4x2 `t2m(lat, lon)` grid on an uneven latitude stored south to north, at the root
/// and in group `g1`, with values `0..8` in storage order.
fn grid_file(nc: &TempNc) -> super::desktop::NetCdfBlockStore {
    let _ = inspect(nc, |f| {
        f.add_dimension("lat", 4).expect("lat");
        f.add_dimension("lon", 2).expect("lon");
        put(f, "lat", &["lat"], &[-60.0f64, -20.0, 0.0, 50.0]);
        put(f, "lon", &["lon"], &[0.0f64, 10.0]);
        let values: Vec<f32> = (0..8u8).map(f32::from).collect();
        put(f, "t2m", &["lat", "lon"], &values);
        f.add_group("g1").expect("g1");
        put(f, "g1/t2m", &["lat", "lon"], &values);
        // The group's own longitudes shadow the root's for its variables.
        put(f, "g1/lon", &["lon"], &[100.0f64, 110.0]);
    });
    super::desktop::NetCdfBlockStore::open_local(nc.path()).expect("open")
}

#[test]
fn uneven_south_to_north_netcdf_rows_hover_with_their_own_latitude() {
    use crate::data::blocks::BlockStore;
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_hover");
    let store = grid_file(&nc);
    let meta = store.inspect().expect("inspect");
    let request = crate::data::slice_request::SliceRequest::full_range("t2m", &[4, 2]);
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(
        block.coordinates["lat"],
        [50.0, 0.0, -20.0, -60.0],
        "rows run north first"
    );

    let idx = meta
        .variables
        .iter()
        .position(|v| v.name == "t2m")
        .expect("t2m");
    let mut app = crate::app::OctantApp {
        plotted_dataset_metadata: Some(meta),
        plotted_variable_idx: idx,
        ..Default::default()
    };
    app.apply_2d_projection(&block, 1, 0, (0, 2), (0, 4), &[0, 0], true, 0);
    let meta = app.plotted_dataset_metadata.as_ref();
    let var = meta.and_then(|m| m.variables.get(idx));
    let matrix = app.matrix_data.as_ref().expect("matrix");
    let (val, fields, _, _) = crate::ui::hover::entries_2d::resolve_2d_plot_entries(
        &app, matrix, meta, var, 0.2, 0.05, None,
    );
    let lat = fields
        .iter()
        .find(|f| &*f.label == "lat")
        .map(|f| f.value.as_str());
    assert_eq!((val, lat), (6.0, Some("50.00°N")));
}

#[test]
fn grouped_blocks_find_coordinates_up_their_group_chain() {
    use crate::data::blocks::BlockStore;
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_group_block");
    let store = grid_file(&nc);
    let request = crate::data::slice_request::SliceRequest::full_range("g1/t2m", &[4, 2]);
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(block.coordinates["lat"], [50.0, 0.0, -20.0, -60.0]);
    assert_eq!(
        block.coordinates["lon"],
        [100.0, 110.0],
        "g1/lon, not the root's"
    );
    let meta = store.inspect().expect("inspect");
    let lon = meta
        .get_dim_coords(Some("g1/t2m"), "lon")
        .and_then(|c| c.number(1));
    assert_eq!(lon, Some(110.0));
}

#[test]
fn a_one_row_netcdf_edge_block_flips_with_its_whole_latitude() {
    use crate::data::blocks::BlockStore;
    use crate::data::slice_request::{DimensionSelection, SliceRequest};
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_edge_block");
    let store = grid_file(&nc);
    let request = SliceRequest::new(
        "t2m",
        vec![
            DimensionSelection::range(3, 4),
            DimensionSelection::range(0, 2),
        ],
    );
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(
        block.flipped_dims,
        ["lat"],
        "decided from the whole latitude"
    );
    assert_eq!(block.shape, [1, 2]);
}

#[test]
fn a_text_coordinate_in_the_group_hides_the_roots_numbers() {
    use crate::data::blocks::BlockStore;
    let _lock = netcdf_lock();
    let nc = TempNc::new("coord_group_text");
    let _ = inspect(&nc, |f| {
        f.add_dimension("lat", 2).expect("lat");
        f.add_dimension("lon", 2).expect("lon");
        put(f, "lat", &["lat"], &[10.0f64, 0.0]);
        put(f, "lon", &["lon"], &[0.0f64, 10.0]);
        let mut g = f.add_group("g").expect("g");
        let mut lon = g.add_string_variable("lon", &["lon"]).expect("g/lon");
        for (i, name) in ["west", "east"].iter().enumerate() {
            lon.put_string(name, [i]).expect("lon label");
        }
        put(f, "g/t2m", &["lat", "lon"], &[0.0f32, 1.0, 2.0, 3.0]);
    });
    let store = super::desktop::NetCdfBlockStore::open_local(nc.path()).expect("open");
    let request = crate::data::slice_request::SliceRequest::full_range("g/t2m", &[2, 2]);
    let block = store.fetch_block(&request).expect("block");
    assert_eq!(block.coordinates["lat"], [10.0, 0.0]);
    assert!(
        !block.coordinates.contains_key("lon"),
        "the group's labels, not the root's numbers"
    );
}
