//! Dataset Catalog Subsystem providing built-in remote and procedural data presets.

pub mod entries;
pub mod microscopy;
pub mod types;

#[cfg(test)]
mod tests;

pub use entries::{GEOTIFF_CATALOG, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG};
pub use microscopy::MICROSCOPY_CATALOG;
pub use types::{CatalogCategoryFilter, CatalogEntry, CatalogProvider, StaticCatalogProvider};

/// Every entry of every category.
pub fn total_entries() -> usize {
    ZARR_CATALOG.len()
        + ICECHUNK_CATALOG.len()
        + GEOTIFF_CATALOG.len()
        + MICROSCOPY_CATALOG.len()
        + PROCEDURAL_CATALOG.len()
}

/// Returns all static catalog entries matching the selected category filter.
pub fn get_catalog_entries(filter: CatalogCategoryFilter) -> Vec<&'static CatalogEntry> {
    match filter {
        CatalogCategoryFilter::All => ZARR_CATALOG
            .iter()
            .chain(ICECHUNK_CATALOG.iter())
            .chain(GEOTIFF_CATALOG.iter())
            .chain(MICROSCOPY_CATALOG.iter())
            .chain(PROCEDURAL_CATALOG.iter())
            .collect(),
        CatalogCategoryFilter::Zarr => ZARR_CATALOG.iter().collect(),
        CatalogCategoryFilter::Icechunk => ICECHUNK_CATALOG.iter().collect(),
        CatalogCategoryFilter::GeoTiff => GEOTIFF_CATALOG.iter().collect(),
        CatalogCategoryFilter::Microscopy => MICROSCOPY_CATALOG.iter().collect(),
        CatalogCategoryFilter::Procedural => PROCEDURAL_CATALOG.iter().collect(),
    }
}
