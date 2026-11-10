//! Desktop stores read again from their source once their dataset closes (B7).

use bytes::Bytes;
use icechunk::format::Path;
use zarrs::storage::{ReadableStorageTraits, StoreKey};

use super::geotiff::store::GeoTiffBlockStore;
use super::geotiff::test_utils::SyntheticTiffBuilder;
use super::icechunk::build_sync_icechunk_store;
use super::release_stores;
use crate::data::blocks::BlockStore;

#[test]
fn a_rewritten_tiff_opens_at_its_new_size() {
    let path = std::env::temp_dir().join(format!("octant_release_{}.tif", std::process::id()));
    let uri = path.to_str().expect("path");
    let write = |side: u32| {
        let bytes = SyntheticTiffBuilder::new(side, side)
            .striped_data(&vec![1u8; (side * side) as usize], side)
            .build();
        std::fs::write(&path, bytes).expect("write tiff");
    };
    let shape = || {
        let meta = GeoTiffBlockStore::open(uri).expect("open").inspect();
        let meta = meta.expect("inspect");
        let band = meta.variables.iter().find(|v| v.name == "band_1");
        band.expect("band_1").shape.clone()
    };
    write(4);
    assert_eq!(shape(), [4, 4]);
    release_stores(uri);
    write(6);
    assert_eq!(shape(), [6, 6], "read again after release");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_released_icechunk_repo_opens_at_its_new_commit() {
    let dir = std::env::temp_dir().join(format!("octant_release_{}.icechunk", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let location = dir.to_str().expect("path");
    let group = Bytes::from_static(br#"{"zarr_format":3,"node_type":"group"}"#);
    let commit = |path: &str| {
        crate::utils::executor::get_shared_tokio_rt().block_on(async {
            let storage = icechunk::new_local_filesystem_storage(&dir)
                .await
                .expect("storage");
            let repo = match path {
                "/" => icechunk::Repository::create(storage).execute().await,
                _ => icechunk::Repository::open(storage).execute().await,
            }
            .expect("repo");
            let mut session = repo.writable_session("main").await.expect("session");
            let at = Path::try_from(path.to_string()).expect("path");
            session.add_group(at, group.clone()).await.expect("group");
            session.commit("test").execute().await.expect("commit");
        })
    };
    let has_new = || {
        let store = build_sync_icechunk_store(location).expect("open");
        store
            .get(&StoreKey::new("new/zarr.json").expect("key"))
            .ok()
            .flatten()
            .is_some()
    };
    commit("/");
    assert!(!has_new());
    release_stores(location);
    commit("/new");
    assert!(has_new(), "the new commit after release");
    let _ = std::fs::remove_dir_all(&dir);
}
