//! Universal async reader implementations for TIFF/GeoTIFF reading.

use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use async_tiff::error::AsyncTiffError;
use async_tiff::error::AsyncTiffResult;
use async_tiff::reader::AsyncFileReader;
use async_trait::async_trait;
use bytes::Bytes;

/// In-memory async reader for TIFF byte buffers (useful for testing and drag-and-drop bytes).
#[derive(Debug, Clone)]
pub struct MemoryTiffReader {
    data: Bytes,
}

impl MemoryTiffReader {
    pub fn new(data: impl Into<Bytes>) -> Self {
        Self { data: data.into() }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl AsyncFileReader for MemoryTiffReader {
    async fn get_bytes(&self, range: Range<u64>) -> AsyncTiffResult<Bytes> {
        let start = (range.start as usize).min(self.data.len());
        let end = (range.end as usize).min(self.data.len());
        if start >= end {
            return Ok(Bytes::new());
        }
        Ok(self.data.slice(start..end))
    }
}

/// WASM HTTP byte range reader backed by browser `window.fetch`.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct WasmHttpTiffReader {
    url: String,
}

#[cfg(target_arch = "wasm32")]
impl WasmHttpTiffReader {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl AsyncFileReader for WasmHttpTiffReader {
    async fn get_bytes(&self, range: Range<u64>) -> AsyncTiffResult<Bytes> {
        let len = range.end.saturating_sub(range.start);
        let bytes = crate::data::backends::http::fetch_url_byte_range(&self.url, range.start, len)
            .await
            .map_err(|e| AsyncTiffError::General(e))?;
        Ok(Bytes::from(bytes))
    }
}

/// Creates an `Arc<dyn AsyncFileReader>` for a given URI or local path.
pub async fn create_async_reader(
    uri: &str,
) -> Result<Arc<dyn AsyncFileReader>, crate::data::blocks::BlockStoreError> {
    let clean = uri.trim();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if clean.starts_with("http://") || clean.starts_with("https://") {
            let parsed_url =
                reqwest::Url::parse(clean).map_err(|e| format!("Invalid URL '{clean}': {e}"))?;
            let client = reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default();
            let reader = async_tiff::reader::ReqwestReader::new(client, parsed_url);
            Ok(Arc::new(reader))
        } else {
            let file_path = if let Some(stripped) = clean.strip_prefix("file://") {
                stripped
            } else {
                clean
            };
            let file = tokio::fs::File::open(file_path)
                .await
                .map_err(|e| format!("Failed to open TIFF file at '{file_path}': {e}"))?;
            let reader = async_tiff::reader::TokioReader::new(file);
            Ok(Arc::new(reader))
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        if clean.starts_with("http://") || clean.starts_with("https://") {
            Ok(Arc::new(WasmHttpTiffReader::new(clean)))
        } else {
            Err(
                format!("Local filesystem paths are not supported directly in WASM: {clean}")
                    .into(),
            )
        }
    }
}
