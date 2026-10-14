//! Reading a plate's wells in the background: each well's attributes, from
//! disk or over HTTP, sent back once (a read that stops sends a failure).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use crate::data::ome::OmeWell;
use crate::i18n::{NoticesKey, filled, tr};

/// Whether a plate's wells answered as Zarr v2 (`.zattrs`): its other wells
/// are then asked for `.zattrs` first. Shared by the plate's reads.
pub(super) type ZarrV2 = Arc<AtomicBool>;

/// `key`'s notice about the well at `path`.
fn well_error(key: NoticesKey, path: &str) -> String {
    filled(tr(key), &[&path])
}

/// The well group's attributes: `zarr.json` (Zarr v3) or `.zattrs` (Zarr v2),
/// the plate's format first once a well of it answered.
async fn well_attributes(
    url: &str,
    path: &str,
    zarr_v2: &AtomicBool,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let v2 = zarr_v2.load(Ordering::Relaxed);
    let names = if v2 {
        [".zattrs", "zarr.json"]
    } else {
        ["zarr.json", ".zattrs"]
    };
    let mut last_error = String::new();
    for name in names {
        let key = format!("{url}/{path}/{name}");
        match read_bytes(&key).await {
            Ok(bytes) => {
                let (attrs, format) = crate::data::ome::reader::attributes_of(&bytes)
                    .ok_or_else(|| well_error(NoticesKey::PlateWellNotJson, path))?;
                zarr_v2.store(format == 2, Ordering::Relaxed);
                return Ok(attrs);
            }
            Err(e) => last_error = format!("{path}: {e}"),
        }
    }
    Err(last_error)
}

/// Bytes at `location`: fetched over HTTP, or read from disk (desktop).
async fn read_bytes(location: &str) -> Result<Vec<u8>, String> {
    if location.contains("://") {
        return crate::data::backends::http::fetch_url_bytes(location).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read(location).map_err(|e| e.to_string())
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(crate::i18n::tr(crate::i18n::DatasetKey::BrowserLocalPaths).to_owned())
    }
}

/// The fields of the well at `path` of the plate at `url`.
pub(super) async fn read_well(
    url: String,
    path: String,
    zarr_v2: ZarrV2,
) -> Result<OmeWell, String> {
    let attrs = well_attributes(&url, &path, &zarr_v2).await?;
    let parsed = crate::data::ome::parse::parse_group(attrs, &|_| None)
        .ok_or_else(|| well_error(NoticesKey::PlateWellNotOme, &path))?;
    let well = parsed
        .fields
        .well
        .ok_or_else(|| well_error(NoticesKey::PlateWellNoMetadata, &path))?;
    Ok(crate::data::ome::plate::well_info(well))
}

pub(super) type WellResult = (String, Result<OmeWell, String>);

/// Sends a well's result once; a read that ends without one (a panic)
/// sends a failure, so the well never stays loading.
pub(super) struct Reply {
    pub(super) path: String,
    pub(super) tx: Option<Sender<WellResult>>,
}

impl Reply {
    fn send(mut self, result: Result<OmeWell, String>) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send((std::mem::take(&mut self.path), result));
        }
    }
}

impl Drop for Reply {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let error = well_error(NoticesKey::PlateWellStopped, &self.path);
            let _ = tx.send((std::mem::take(&mut self.path), Err(error)));
        }
    }
}

pub(super) fn fetch_well(url: String, path: String, zarr_v2: ZarrV2, tx: Sender<WellResult>) {
    let reply = Reply {
        path: path.clone(),
        tx: Some(tx),
    };
    #[cfg(not(target_arch = "wasm32"))]
    crate::utils::executor::TaskExecutor::spawn_background(move || {
        let rt = crate::utils::executor::get_shared_tokio_rt();
        reply.send(rt.block_on(read_well(url, path, zarr_v2)));
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        reply.send(read_well(url, path, zarr_v2).await);
    });
}
