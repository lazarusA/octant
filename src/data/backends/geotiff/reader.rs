//! Universal async reader implementations for TIFF/GeoTIFF reading.

use std::fmt::Debug;
use std::ops::Range;
use std::sync::Arc;

use async_tiff::error::AsyncTiffResult;
use async_tiff::reader::AsyncFileReader;
use async_trait::async_trait;
use bytes::Bytes;

/// In-memory async reader for TIFF byte buffers.
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

/// Robust Tokio async file reader pre-allocating buffer space.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct TokioFileReader {
    file: tokio::sync::Mutex<tokio::fs::File>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TokioFileReader {
    pub fn new(file: tokio::fs::File) -> Self {
        Self {
            file: tokio::sync::Mutex::new(file),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl AsyncFileReader for TokioFileReader {
    async fn get_bytes(&self, range: Range<u64>) -> AsyncTiffResult<Bytes> {
        use std::io::SeekFrom;
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(range.start)).await?;
        let to_read = (range.end.saturating_sub(range.start)) as usize;
        let mut buffer = Vec::with_capacity(to_read);
        (&mut *file)
            .take(to_read as u64)
            .read_to_end(&mut buffer)
            .await?;
        Ok(Bytes::from(buffer))
    }
}

/// Universal HTTP byte range reader backed by `fetch_url_byte_range`.
#[derive(Debug, Clone)]
pub struct WasmHttpTiffReader {
    url: String,
}

impl WasmHttpTiffReader {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl AsyncFileReader for WasmHttpTiffReader {
    async fn get_bytes(&self, range: Range<u64>) -> AsyncTiffResult<Bytes> {
        let len = range.end.saturating_sub(range.start);
        let bytes = crate::data::backends::http::fetch_url_byte_range(&self.url, range.start, len)
            .await
            .map_err(async_tiff::error::AsyncTiffError::General)?;
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
            let parsed =
                reqwest::Url::parse(clean).map_err(|e| format!("Invalid URL '{clean}': {e}"))?;
            let client = crate::data::backends::http::get_http_client().clone();
            Ok(Arc::new(async_tiff::reader::ReqwestReader::new(
                client, parsed,
            )))
        } else {
            let path = clean.strip_prefix("file://").unwrap_or(clean);
            let file = tokio::fs::File::open(path)
                .await
                .map_err(|e| format!("Failed to open TIFF at '{path}': {e}"))?;
            Ok(Arc::new(TokioFileReader::new(file)))
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        if clean.starts_with("http://") || clean.starts_with("https://") {
            Ok(Arc::new(WasmHttpTiffReader::new(clean)))
        } else {
            Err(format!("Local filesystem paths unsupported on WASM: {clean}").into())
        }
    }
}
