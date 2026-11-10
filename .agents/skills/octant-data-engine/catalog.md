# Catalog (`src/catalog/`)

Reference for this skill, moved from `AGENTS.md` (which keeps the rules most tasks need); update it with the code it describes.

- **Catalog Subsystem (`src/catalog/`)**:
  - `entries.rs`: Built-in static catalog tables (Zarr, Icechunk, Procedural).
  - `types.rs`: `CatalogEntry`, `CatalogCategoryFilter`. Entries are `&'static` tables; a fetched catalog (STAC, a JSON manifest) would need owned entries loaded in the background and merged with these tables in `get_catalog_entries` (the earlier `CatalogProvider` trait returned `&'static` entries, so it could not hold one, and was removed).
  - `geotiff.rs`: the GeoTIFF category's COGs: `GEOTIFF_CATALOG` (servers that send CORS headers) and `GEOTIFF_DESKTOP_CATALOG` (servers that do not; `CatalogEntry::desktop_only`, shown in the browser with a "Desktop only" chip by `ui/catalog/card.rs`; in the browser their load button explains instead of loading, `app/sources.rs` `open_catalog_entry`, the one way an entry opens: its card, voice commands).
  - `soil.rs`: `SOIL_CATALOG`, the Soil category's soil property COGs (ISRIC SoilGrids 2.0, OpenLandMap-soildb, OpenLandMap soil classes), all readable in the browser.
  - `microscopy.rs`: `MICROSCOPY_CATALOG`, the Microscopy category's public OME-Zarr datasets (IDR, EBI, JAX, Janelia, NFDI4BIOIMAGE at Münster and Göttingen, RIKEN SSBD; shown with an OME-Zarr badge).
  - `tests.rs`: Category totals, unique keys, Microscopy entries open as remote Zarr, Soil entries as remote GeoTIFF, only `GEOTIFF_DESKTOP_CATALOG` entries are desktop-only. `data/backends/geotiff/network_tests.rs` (`cargo test --lib geotiff_catalog_entries_open -- --ignored --nocapture`) opens every GeoTIFF and Soil entry and checks each server's CORS answer matches `desktop_only`.
  - `mod.rs`: Re-exports and `get_catalog_entries`.
