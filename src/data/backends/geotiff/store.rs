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

use super::decode::create_decoder_registry;
use super::inspect::inspect_tiff;
use super::reader::{MemoryTiffReader, create_async_reader};
use super::slicing::fetch_geotiff_block;

#[cfg(not(target_arch = "wasm32"))]
static GEOTIFF_STORE_CACHE: std::sync::OnceLock<
    std::sync::RwLock<std::collections::HashMap<String, GeoTiffBlockStore>>,
> = std::sync::OnceLock::new();

/// BlockStore for reading tiled and striped TIFF/GeoTIFF raster datasets.
#[derive(Clone)]
pub struct GeoTiffBlockStore {
    pub(crate) uri: String,
    pub(crate) reader: Arc<dyn AsyncFileReader>,
    pub(crate) tiff: TIFF,
    pub(crate) metadata: DatasetMetadata,
    pub(crate) decoder_registry: Arc<DecoderRegistry>,
}

impl GeoTiffBlockStore {
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn tiff(&self) -> &TIFF {
        &self.tiff
    }

    /// Open a TIFF/GeoTIFF dataset from a URI (file path or HTTP URL).
    pub fn open(uri: &str) -> Result<Self, BlockStoreError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let cache_lock = GEOTIFF_STORE_CACHE
                .get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));
            let cache = cache_lock.read().unwrap_or_else(|p| p.into_inner());
            if let Some(store) = cache.get(uri) {
                return Ok(store.clone());
            }
            drop(cache);

            let rt = crate::utils::executor::get_shared_tokio_rt();
            let u = uri.to_string();
            let store = rt.block_on(async move { Self::open_async(&u).await })?;

            let mut cache = cache_lock.write().unwrap_or_else(|p| p.into_inner());
            cache.insert(uri.to_string(), store.clone());
            Ok(store)
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            let cache_lock = GEOTIFF_STORE_CACHE
                .get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));
            {
                let cache = cache_lock.read().unwrap_or_else(|p| p.into_inner());
                if let Some(store) = cache.get(uri) {
                    return Ok(store.clone());
                }
            }

            let reader = create_async_reader(uri).await?;
            let store = Self::from_reader(uri, reader).await?;

            let mut cache = cache_lock.write().unwrap_or_else(|p| p.into_inner());
            cache.insert(uri.to_string(), store.clone());
            Ok(store)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let reader = create_async_reader(uri).await?;
            Self::from_reader(uri, reader).await
        }
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
        let cached = ReadaheadMetadataCache::new(reader.clone());
        let mut meta_reader = TiffMetadataReader::try_open(&cached)
            .await
            .map_err(|e| format!("Failed to read TIFF header for '{name}': {e}"))?;
        let ifds = meta_reader
            .read_all_ifds(&cached)
            .await
            .map_err(|e| format!("Failed to read IFD metadata for '{name}': {e}"))?;
        if ifds.is_empty() {
            return Err(format!("TIFF '{name}' contains no IFDs").into());
        }
        let tiff = TIFF::new(ifds, meta_reader.endianness());
        let metadata = inspect_tiff(&tiff, name);
        let decoder_registry = Arc::new(create_decoder_registry());
        Ok(Self {
            uri: name.into(),
            reader,
            tiff,
            metadata,
            decoder_registry,
        })
    }

    pub(crate) fn resolve_ifd(&self, var: &str) -> Option<&async_tiff::ImageFileDirectory> {
        if var.starts_with("overview_") {
            let idx = var
                .strip_prefix("overview_")?
                .split('/')
                .next()?
                .parse::<usize>()
                .ok()?;
            self.tiff.ifds().get(idx)
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
        req: &SliceRequest,
        _on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        let ifd = self
            .resolve_ifd(&req.variable)
            .ok_or_else(|| format!("Variable '{}' not found in TIFF", req.variable))?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let rt = crate::utils::executor::get_shared_tokio_rt();
            rt.block_on(async {
                fetch_geotiff_block(
                    ifd,
                    self.tiff.endianness(),
                    req,
                    self.reader.as_ref(),
                    &self.decoder_registry,
                )
                .await
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            futures::executor::block_on(async {
                fetch_geotiff_block(
                    ifd,
                    self.tiff.endianness(),
                    req,
                    self.reader.as_ref(),
                    &self.decoder_registry,
                )
                .await
            })
        }
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let blocks = requests
            .iter()
            .map(|r| self.fetch_block(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(BlockResult::new(blocks))
    }
}
