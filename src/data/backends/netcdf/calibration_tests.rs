//! CF calibration read alike by every NetCDF reader and by Zarr.

use netcdf::Options;

use super::test_support::{Parity, TempNc, netcdf_lock};
use crate::data::blocks::BlockStore;
use crate::data::slice_request::SliceRequest;

/// `valid_range` masks values outside it, in classic and netCDF-4 files, as Zarr's
/// attributes do (B13).
#[test]
fn valid_range_masks_values_outside_it() {
    let _lock = netcdf_lock();
    for (name, options) in [("range4", Options::NETCDF4), ("range1", Options::empty())] {
        let nc = TempNc::new(name);
        {
            let mut f = netcdf::create_with(&nc.0, options).expect("create");
            f.add_dimension("x", 4).expect("x");
            let mut v = f.add_variable::<f32>("v", &["x"]).expect("v");
            v.put_attribute("valid_range", vec![0.0f32, 10.0])
                .expect("valid_range");
            // Classic files take every definition before any data.
            f.enddef().expect("end of definitions");
            let mut v = f.variable_mut("v").expect("v");
            v.put_values(&[1.0f32, 5.0, 10.0, 20.0], ..)
                .expect("values");
        }
        let store = Parity::open(nc.path()).expect("open");
        let block = store
            .fetch_block(&SliceRequest::full_range("v", &[4]))
            .expect("block");
        let masked = block.values.iter().map(|v| v.is_nan()).collect::<Vec<_>>();
        assert_eq!(
            masked,
            [false, false, false, true],
            "{name}: {:?}",
            block.values
        );
    }
}
