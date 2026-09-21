//! Dataset Catalog Subsystem providing built-in remote and procedural data presets.

pub mod entries;
pub mod types;

pub use entries::{GEOTIFF_CATALOG, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG};
pub use types::{CatalogCategoryFilter, CatalogEntry, CatalogProvider, StaticCatalogProvider};

/// Returns all static catalog entries matching the selected category filter.
pub fn get_catalog_entries(filter: CatalogCategoryFilter) -> Vec<&'static CatalogEntry> {
    match filter {
        CatalogCategoryFilter::All => ZARR_CATALOG
            .iter()
            .chain(ICECHUNK_CATALOG.iter())
            .chain(GEOTIFF_CATALOG.iter())
            .chain(PROCEDURAL_CATALOG.iter())
            .collect(),
        CatalogCategoryFilter::Zarr => ZARR_CATALOG.iter().collect(),
        CatalogCategoryFilter::Icechunk => ICECHUNK_CATALOG.iter().collect(),
        CatalogCategoryFilter::GeoTiff => GEOTIFF_CATALOG.iter().collect(),
        CatalogCategoryFilter::Procedural => PROCEDURAL_CATALOG.iter().collect(),
    }
}
