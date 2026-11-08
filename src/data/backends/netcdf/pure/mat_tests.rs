use hdf5_pure::mat::{MatBuilder, Options};

use crate::data::backends::netcdf::NetCdfBlockStore;
use crate::data::backends::netcdf::test_support::TempNc;
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;

#[test]
fn matlab_files_open_with_their_numeric_arrays() {
    let mat = TempNc::new("matlab.mat");
    let mut mb = MatBuilder::new(Options::default());
    // The 2x3 matrix [1 2 3; 4 5 6], column-major.
    mb.write_f64("a", &[2, 3], &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0])
        .expect("a");
    mb.struct_("s", |s| s.write_scalar_f32("x", 7.5).map(|_| ()))
        .expect("s");
    mb.write_char("label", "text").expect("label");
    mb.cell("c", &[1, 1], |c| c.push_scalar_f64(1.0).map(|_| ()))
        .expect("c");
    mb.write(&mat.0).expect("write");

    let store = NetCdfBlockStore::open_local(mat.path()).expect("open");
    let meta = store.inspect().expect("inspect");
    let mut names: Vec<&str> = meta.variables.iter().map(|v| v.name.as_str()).collect();
    names.sort();
    assert_eq!(names, ["a", "s/x"]);
    let a = meta.variables.iter().find(|v| v.name == "a").expect("a");
    assert_eq!(a.shape, [3, 2]);
    assert_eq!(a.dimension_names, ["dim_2", "dim_1"]);

    let block = store
        .fetch_block(&SliceRequest::full_range("a", &[3, 2]))
        .expect("block");
    assert_eq!(&*block.values, [1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn matlab_files_before_v7_3_are_refused_with_the_way_out() {
    let mat = TempNc::new("old.mat");
    let mut head = b"MATLAB 5.0 MAT-file, Platform: x".to_vec();
    head.resize(256, b' ');
    std::fs::write(&mat.0, head).expect("write");
    let error = NetCdfBlockStore::open_local(mat.path()).expect_err("refused");
    assert!(error.to_string().contains("-v7.3"), "{error}");
}
