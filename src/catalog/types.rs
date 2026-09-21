//! Catalog entry types and provider traits.

use crate::app::StoreKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub key: &'static str,
    pub label: &'static str,
    pub subtitle: &'static str,
    pub store: &'static str,
    pub store_kind: StoreKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogCategoryFilter {
    All,
    Zarr,
    Icechunk,
    GeoTiff,
    Procedural,
}

/// Standard interface for dataset catalog providers (static lists, STAC APIs, JSON manifests).
pub trait CatalogProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn fetch_entries(&self, filter: CatalogCategoryFilter) -> Vec<&'static CatalogEntry>;
}

pub struct StaticCatalogProvider;

impl CatalogProvider for StaticCatalogProvider {
    fn name(&self) -> &'static str {
        "Static Built-in Catalog"
    }

    fn fetch_entries(&self, filter: CatalogCategoryFilter) -> Vec<&'static CatalogEntry> {
        super::get_catalog_entries(filter)
    }
}
