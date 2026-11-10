//! The browser's GeoTIFF store reads a file's headers once (B6), run natively on Tokio
//! against a local range server.

use std::collections::HashMap;
use std::sync::Arc;

use super::inspect_wasm_remote_geotiff;
use crate::data::backends::geotiff::reader::WasmHttpTiffReader;
use crate::data::backends::geotiff::store::GeoTiffBlockStore;
use crate::data::backends::geotiff::test_utils::SyntheticTiffBuilder;
use crate::data::backends::zarr::wasm::range_server::RangeServer;

#[test]
fn a_cold_inspection_reads_the_headers_once() {
    let tiff = SyntheticTiffBuilder::new(4, 4)
        .striped_data(&[1u8; 16], 4)
        .build();
    let files = HashMap::from([("a.tif".into(), tiff.clone()), ("b.tif".into(), tiff)]);
    let server = RangeServer::start(files);
    let url = |name: &str| format!("{}/{name}", server.base);
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let reader = Arc::new(WasmHttpTiffReader::new(url("a.tif")));
    rt.block_on(GeoTiffBlockStore::from_reader(&url("a.tif"), reader))
        .expect("open");
    let one_open = server.requests();
    rt.block_on(inspect_wasm_remote_geotiff(&url("b.tif")))
        .expect("inspect");
    assert_eq!(server.requests() - one_open, one_open, "headers read once");
}

/// Opens asked while one reads wait for it, and a failed open is read again by the next.
#[test]
fn opens_at_once_read_once_and_failures_read_again() {
    use super::WasmGeoTiffBlockStore;
    let tiff = SyntheticTiffBuilder::new(4, 4)
        .striped_data(&[1u8; 16], 4)
        .build();
    let files = HashMap::from([("c.tif".into(), tiff.clone()), ("d.tif".into(), tiff)]);
    let server = RangeServer::start(files);
    let store =
        |name: &str| WasmGeoTiffBlockStore::get_or_create(&format!("{}/{name}", server.base));
    let rt = crate::utils::executor::get_shared_tokio_rt();

    let missing = store("none.tif");
    assert!(rt.block_on(missing.open()).is_err());
    let after_first = server.requests();
    assert!(rt.block_on(missing.open()).is_err());
    assert!(
        server.requests() > after_first,
        "a failed open is read again"
    );

    let before = server.requests();
    rt.block_on(store("d.tif").open()).expect("open d");
    let one_open = server.requests() - before;
    let c = store("c.tif");
    let before = server.requests();
    let (a, b) = rt.block_on(futures::future::join(c.open(), c.open()));
    assert!(a.is_ok() && b.is_ok());
    assert_eq!(server.requests() - before, one_open, "two opens, one read");
}
