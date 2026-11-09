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
    /// Soil property maps (Cloud-Optimized GeoTIFFs).
    Soil,
    /// Bioimaging OME-Zarr datasets.
    Microscopy,
    Procedural,
}
