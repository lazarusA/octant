//! WebAssembly asynchronous block preloader for GeoTIFF datasets.

use crate::data::blocks::{BlockRequest, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;

#[cfg(target_arch = "wasm32")]
use super::super::reader::WasmHttpTiffReader;
#[cfg(target_arch = "wasm32")]
use super::super::store::GeoTiffBlockStore;
#[cfg(target_arch = "wasm32")]
use super::store::WasmGeoTiffBlockStore;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

/// Asynchronously loads one block on WASM for a remote GeoTIFF dataset.
#[cfg(target_arch = "wasm32")]
pub async fn load_one_geotiff_wasm_with_progress(
    request: &BlockRequest,
    _on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    let source_uri = &request.store.source().uri;
    let store = WasmGeoTiffBlockStore::get_or_create(source_uri);

    let cached = {
        let guard = store.inner.read().unwrap_or_else(|p| p.into_inner());
        guard.as_ref().map(|s| {
            (
                s.resolve_ifd(&request.slice.variable).cloned(),
                s.tiff.endianness(),
                s.reader.clone(),
                s.decoder_registry.clone(),
            )
        })
    };

    let (ifd, endianness, reader, decoder_registry) = match cached {
        Some((Some(ifd), endianness, reader, decoder_registry)) => {
            (ifd, endianness, reader, decoder_registry)
        }
        Some((None, _, _, _)) => {
            return Err(
                format!("Variable '{}' not found in GeoTIFF", request.slice.variable).into(),
            );
        }
        None => {
            let reader = Arc::new(WasmHttpTiffReader::new(source_uri));
            let geo_store = GeoTiffBlockStore::from_reader(source_uri, reader)
                .await
                .map_err(|e| e.to_string())?;
            let ifd = geo_store
                .resolve_ifd(&request.slice.variable)
                .cloned()
                .ok_or_else(|| {
                    format!("Variable '{}' not found in GeoTIFF", request.slice.variable)
                })?;
            let endianness = geo_store.tiff.endianness();
            let r = geo_store.reader.clone();
            let dec = geo_store.decoder_registry.clone();
            let mut guard = store.inner.write().unwrap_or_else(|p| p.into_inner());
            *guard = Some(geo_store);
            (ifd, endianness, r, dec)
        }
    };

    super::super::slicing::fetch_geotiff_block(
        &ifd,
        endianness,
        &request.slice,
        reader.as_ref(),
        &decoder_registry,
    )
    .await
}

/// Asynchronously loads one block on desktop fallback.
#[cfg(not(target_arch = "wasm32"))]
pub async fn load_one_geotiff_wasm_with_progress(
    request: &BlockRequest,
    on_progress: ProgressCallback<'_>,
) -> Result<OctantBlock, BlockStoreError> {
    request
        .store
        .fetch_with_progress(&request.slice, on_progress)
}
