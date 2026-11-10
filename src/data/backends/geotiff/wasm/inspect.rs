//! WebAssembly remote COG metadata inspection.

use crate::data::DatasetMetadata;
use crate::data::blocks::BlockStore;

use super::store::WasmGeoTiffBlockStore;

/// Inspects a remote GeoTIFF/COG URL in the browser, opening its store once.
pub async fn inspect_wasm_remote_geotiff(url: &str) -> Result<DatasetMetadata, String> {
    let clean_url = url.trim();
    if clean_url.is_empty() {
        return Err("URL is empty".to_string());
    }
    let store = WasmGeoTiffBlockStore::get_or_create(clean_url)
        .open()
        .await?;
    store.inspect().map_err(|e| e.to_string())
}
