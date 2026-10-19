//! Catalog categories and entries.

use super::{CatalogCategoryFilter, get_catalog_entries, total_entries};
use crate::app::StoreKind;

const CATEGORIES: [CatalogCategoryFilter; 6] = [
    CatalogCategoryFilter::Zarr,
    CatalogCategoryFilter::Icechunk,
    CatalogCategoryFilter::GeoTiff,
    CatalogCategoryFilter::Soil,
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

#[test]
fn only_microscopy_entries_are_ome() {
    for &category in &CATEGORIES {
        let ome = category == CatalogCategoryFilter::Microscopy;
        for e in get_catalog_entries(category) {
            assert_eq!(e.is_ome(), ome, "{}", e.key);
        }
    }
}

#[test]
fn soil_entries_are_remote_geotiffs() {
    let entries = get_catalog_entries(CatalogCategoryFilter::Soil);
    assert!(!entries.is_empty());
    for e in entries {
        assert_eq!(e.store_kind, StoreKind::RemoteGeoTiff, "{}", e.key);
        assert!(e.store.starts_with("https://"), "{}", e.key);
        assert_eq!(
            crate::utils::infer_store_kind_from_target(e.store).ok(),
            Some(StoreKind::RemoteGeoTiff),
            "{} opens as GeoTIFF",
            e.key
        );
    }
}

#[test]
fn only_listed_geotiffs_are_desktop_only() {
    let geotiff = get_catalog_entries(CatalogCategoryFilter::GeoTiff);
    for e in &geotiff {
        let listed = super::GEOTIFF_DESKTOP_CATALOG
            .iter()
            .any(|d| d.key == e.key);
        assert_eq!(e.desktop_only(), listed, "{}", e.key);
    }
    assert!(geotiff.iter().any(|e| e.desktop_only()));
    for &category in &CATEGORIES {
        if category != CatalogCategoryFilter::GeoTiff {
            for e in get_catalog_entries(category) {
                assert!(!e.desktop_only(), "{}", e.key);
            }
        }
    }
}
