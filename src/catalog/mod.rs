//! Dataset Catalog Subsystem providing built-in remote and procedural data presets.

pub mod entries;
pub mod geotiff;
pub mod microscopy;
pub mod soil;
pub mod types;

#[cfg(test)]
mod tests;

pub use entries::{ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG};
pub use geotiff::{GEOTIFF_CATALOG, GEOTIFF_DESKTOP_CATALOG};
pub use microscopy::MICROSCOPY_CATALOG;
pub use soil::SOIL_CATALOG;
pub use types::{CatalogCategoryFilter, CatalogEntry, CatalogProvider, StaticCatalogProvider};

/// The GeoTIFF category: browser-readable entries first, then desktop-only ones.
fn geotiff_entries() -> impl Iterator<Item = &'static CatalogEntry> {
    GEOTIFF_CATALOG.iter().chain(GEOTIFF_DESKTOP_CATALOG.iter())
}

/// Entries of the GeoTIFF category.
pub fn geotiff_count() -> usize {
    GEOTIFF_CATALOG.len() + GEOTIFF_DESKTOP_CATALOG.len()
}

/// Every entry of every category.
pub fn total_entries() -> usize {
    ZARR_CATALOG.len()
        + ICECHUNK_CATALOG.len()
        + geotiff_count()
        + SOIL_CATALOG.len()
        + MICROSCOPY_CATALOG.len()
        + PROCEDURAL_CATALOG.len()
}

/// Returns all static catalog entries matching the selected category filter.
pub fn get_catalog_entries(filter: CatalogCategoryFilter) -> Vec<&'static CatalogEntry> {
    match filter {
        CatalogCategoryFilter::All => ZARR_CATALOG
            .iter()
            .chain(ICECHUNK_CATALOG.iter())
            .chain(geotiff_entries())
            .chain(SOIL_CATALOG.iter())
            .chain(MICROSCOPY_CATALOG.iter())
            .chain(PROCEDURAL_CATALOG.iter())
            .collect(),
        CatalogCategoryFilter::Zarr => ZARR_CATALOG.iter().collect(),
        CatalogCategoryFilter::Icechunk => ICECHUNK_CATALOG.iter().collect(),
        CatalogCategoryFilter::GeoTiff => geotiff_entries().collect(),
        CatalogCategoryFilter::Soil => SOIL_CATALOG.iter().collect(),
        CatalogCategoryFilter::Microscopy => MICROSCOPY_CATALOG.iter().collect(),
        CatalogCategoryFilter::Procedural => PROCEDURAL_CATALOG.iter().collect(),
    }
}
