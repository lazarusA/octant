//! Catalog categories and entries.

use super::{CatalogCategoryFilter, get_catalog_entries, total_entries};
use crate::app::StoreKind;

const CATEGORIES: [CatalogCategoryFilter; 5] = [
    CatalogCategoryFilter::Zarr,
    CatalogCategoryFilter::Icechunk,
    CatalogCategoryFilter::GeoTiff,
    CatalogCategoryFilter::Microscopy,
    CatalogCategoryFilter::Procedural,
];

#[test]
fn all_lists_every_category_once() {
    let all = get_catalog_entries(CatalogCategoryFilter::All);
    let summed: usize = CATEGORIES
        .iter()
        .map(|&c| get_catalog_entries(c).len())
        .sum();
    assert_eq!(all.len(), summed);
    assert_eq!(all.len(), total_entries());
}

#[test]
fn keys_are_unique() {
    let all = get_catalog_entries(CatalogCategoryFilter::All);
    let mut keys: Vec<&str> = all.iter().map(|e| e.key).collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), before, "duplicate catalog keys");
}

#[test]
fn microscopy_entries_are_remote_ome_zarr() {
    let entries = get_catalog_entries(CatalogCategoryFilter::Microscopy);
    assert!(!entries.is_empty());
    for e in entries {
        assert_eq!(e.store_kind, StoreKind::RemoteZarr, "{}", e.key);
        assert!(e.store.starts_with("https://"), "{}", e.key);
        assert_eq!(
            crate::utils::infer_store_kind_from_target(e.store).ok(),
            Some(StoreKind::RemoteZarr),
            "{} opens as Zarr",
            e.key
        );
    }
}
