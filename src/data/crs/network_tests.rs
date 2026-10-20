//! CRS discovery on real stores, read over the network.

use super::CrsRef;

/// Real stores through `StoreHandle::inspect`: every level of the Sentinel-2
/// TCI GeoTIFF and EOPF GeoZarr pyramid in UTM 36N, CMIP6 on lon/lat. Run by
/// hand: `cargo test --lib crs_of_real_stores -- --ignored --nocapture`.
#[test]
#[ignore = "reads remote stores"]
fn crs_of_real_stores() {
    use crate::app::StoreKind;
    use crate::data::{DataSource, SourceFactory};
    let open = |kind: StoreKind, target: &str| {
        let id = StoreKind::make_source_id(kind, target);
        let source = DataSource::new(&id, kind.to_data_source_kind(), target, "Store");
        let handle = SourceFactory::open(source).expect("open");
        handle.inspect().expect("inspect")
    };
    let tci = "https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/36/Q/WD/2020/7/S2A_36QWD_20200701_0_L2A/TCI.tif";
    let eopf = crate::catalog::get_catalog_entries(crate::catalog::CatalogCategoryFilter::Zarr)
        .iter()
        .find(|e| e.key == "eopf-s2-l2a-krasnokutsk")
        .expect("EOPF entry")
        .store;
    for (kind, target) in [
        (StoreKind::RemoteGeoTiff, tci),
        (StoreKind::RemoteZarr, eopf),
    ] {
        let meta = open(kind, target);
        for var in meta.variables.iter().filter(|v| v.shape.len() >= 2) {
            assert_eq!(var.crs, Some(CrsRef::Epsg(32636)), "{target}: {}", var.name);
        }
        eprintln!("{target}: {} variables in EPSG:32636", meta.variables.len());
    }
    let cmip6 = "https://storage.googleapis.com/cmip6/CMIP6/CMIP/MPI-M/MPI-ESM1-2-HR/historical/r1i1p1f1/Amon/tas/gn/v20190710/";
    let meta = open(StoreKind::RemoteZarr, cmip6);
    let tas = meta
        .variables
        .iter()
        .find(|v| v.leaf_name() == "tas")
        .expect("tas");
    assert_eq!(tas.crs, Some(CrsRef::Geographic));
}
