//! BlockStore implementation for TIFF and GeoTIFF datasets.

use std::sync::Arc;

use async_tiff::TIFF;
use async_tiff::decoder::DecoderRegistry;
use async_tiff::metadata::TiffMetadataReader;
use async_tiff::metadata::cache::ReadaheadMetadataCache;
use async_tiff::reader::AsyncFileReader;

use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

use super::inspect::inspect_tiff;
use super::reader::{MemoryTiffReader, create_async_reader};
use super::slice::fetch_geotiff_block;

/// BlockStore for reading tiled and striped TIFF/GeoTIFF raster datasets.
pub struct GeoTiffBlockStore {
    uri: String,
    reader: Arc<dyn AsyncFileReader>,
    tiff: TIFF,
    metadata: DatasetMetadata,
    decoder_registry: Arc<DecoderRegistry>,
}

impl GeoTiffBlockStore {
    /// Return the original URI or name used to open this store.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Access the underlying parsed TIFF metadata.
    pub fn tiff(&self) -> &TIFF {
        &self.tiff
    }

    /// Open a TIFF/GeoTIFF dataset from a URI (file path or HTTP URL).
    pub fn open(uri: &str) -> Result<Self, BlockStoreError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let rt = crate::utils::executor::get_shared_tokio_rt();
            let uri_str = uri.to_string();
            rt.block_on(async move { Self::open_async(&uri_str).await })
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(format!(
                "Synchronous opening of remote GeoTIFF is not supported on WASM directly: {uri}"
            )
            .into())
        }
    }

    /// Asynchronously open a TIFF/GeoTIFF dataset from a URI.
    pub async fn open_async(uri: &str) -> Result<Self, BlockStoreError> {
        let reader = create_async_reader(uri).await?;
        Self::from_reader(uri, reader).await
    }

    /// Open a TIFF dataset directly from an in-memory byte buffer.
    pub async fn from_bytes(
        name: &str,
        bytes: impl Into<bytes::Bytes>,
    ) -> Result<Self, BlockStoreError> {
        let reader = Arc::new(MemoryTiffReader::new(bytes));
        Self::from_reader(name, reader).await
    }

    /// Construct a `GeoTiffBlockStore` from an initialized `AsyncFileReader`.
    pub async fn from_reader(
        name: &str,
        reader: Arc<dyn AsyncFileReader>,
    ) -> Result<Self, BlockStoreError> {
        let cached_reader = ReadaheadMetadataCache::new(reader.clone());
        let mut metadata_reader = TiffMetadataReader::try_open(&cached_reader)
            .await
            .map_err(|e| format!("Failed to read TIFF header for '{name}': {e}"))?;

        let ifds = metadata_reader
            .read_all_ifds(&cached_reader)
            .await
            .map_err(|e| format!("Failed to read IFD metadata for '{name}': {e}"))?;

        if ifds.is_empty() {
            return Err(format!("TIFF '{name}' contains no image file directories (IFDs)").into());
        }

        let tiff = TIFF::new(ifds, metadata_reader.endianness());
        let metadata = inspect_tiff(&tiff, name);
        let decoder_registry = Arc::new(super::decompress::create_robust_decoder_registry());

        Ok(Self {
            uri: name.to_string(),
            reader,
            tiff,
            metadata,
            decoder_registry,
        })
    }

    fn resolve_ifd_for_variable(&self, var_name: &str) -> Option<&async_tiff::ImageFileDirectory> {
        if var_name.starts_with("overview_") {
            let ifd_idx = var_name
                .strip_prefix("overview_")?
                .split('/')
                .next()?
                .parse::<usize>()
                .ok()?;
            self.tiff.ifds().get(ifd_idx)
        } else {
            self.tiff.ifds().first()
        }
    }
}

impl BlockStore for GeoTiffBlockStore {
    fn backend_name(&self) -> &str {
        "GeoTIFF"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        Ok(self
            .metadata
            .variables
            .iter()
            .map(|v| v.name.clone())
            .collect())
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        Ok(self.metadata.clone())
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        _on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        let ifd = self
            .resolve_ifd_for_variable(&request.variable)
            .ok_or_else(|| format!("Variable '{}' not found in TIFF", request.variable))?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let rt = crate::utils::executor::get_shared_tokio_rt();
            rt.block_on(async {
                fetch_geotiff_block(ifd, request, self.reader.as_ref(), &self.decoder_registry)
                    .await
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            futures::executor::block_on(async {
                fetch_geotiff_block(ifd, request, self.reader.as_ref(), &self.decoder_registry)
                    .await
            })
        }
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());
        for req in requests {
            blocks.push(self.fetch_block(req)?);
        }
        Ok(BlockResult::new(blocks))
    }
}
