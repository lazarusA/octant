//! Reading a well's fields: Zarr v2 wells have no `zarr.json`.

use super::read_well;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn v2_wells_are_read_after_their_missing_zarr_json() {
    let dir = std::env::temp_dir().join(format!("octant_plate_{}", std::process::id()));
    let well = dir.join("A/1");
    std::fs::create_dir_all(&well).expect("dirs");
    let attrs = r#"{"well": {"version": "0.4", "images": [{"path": "0", "acquisition": 2}, {"path": "1"}]}}"#;
    std::fs::write(well.join(".zattrs"), attrs).expect("attrs");
    let url = dir.to_string_lossy().into_owned();
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let read = rt.block_on(read_well(url.clone(), "A/1".into()));
    let missing = rt.block_on(read_well(url, "B/2".into()));
    let _ = std::fs::remove_dir_all(&dir);
    let well = read.expect("well");
    assert_eq!(well.fields.len(), 2);
    assert_eq!(well.fields[0].acquisition, Some(2));
    assert!(missing.is_err());
}

/// `cargo test --lib idr_plate_well_lists_its_fields -- --ignored`
#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "reads a remote store"]
fn idr_plate_well_lists_its_fields() {
    let url = "https://uk1s3.embassy.ebi.ac.uk/idr/zarr/v0.4/idr0001A/2551.zarr";
    let rt = crate::utils::executor::get_shared_tokio_rt();
    let well = rt
        .block_on(read_well(url.into(), "A/1".into()))
        .expect("well");
    assert_eq!(well.fields.len(), 6);
}
