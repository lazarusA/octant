//! Reading a plate's wells in the background: each well's attributes, from
//! disk or over HTTP, sent back once (a read that stops sends a failure).

use std::collections::BTreeSet;
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use crate::data::ome::OmeWell;

/// Plates whose wells answered as Zarr v2 (`.zattrs`): their other wells
/// are asked for `.zattrs` first.
static ZARR_V2_PLATES: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// The well group's attributes: `zarr.json` (Zarr v3) or `.zattrs` (Zarr v2),
/// the plate's format first once a well of it answered.
async fn well_attributes(
    url: &str,
    path: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let v2 = ZARR_V2_PLATES
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .contains(url);
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
                    .ok_or_else(|| format!("{key}: not a JSON object"))?;
                if (format == 2) != v2 {
                    let mut plates = ZARR_V2_PLATES.lock().unwrap_or_else(|p| p.into_inner());
                    if format == 2 {
                        plates.insert(url.to_string());
                    } else {
                        plates.remove(url);
                    }
                }
                return Ok(attrs);
            }
            Err(e) => last_error = e,
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
pub(super) async fn read_well(url: String, path: String) -> Result<OmeWell, String> {
    let attrs = well_attributes(&url, &path).await?;
    let parsed = crate::data::ome::parse::parse_group(attrs, &|_| None)
        .ok_or_else(|| format!("{path}: not an OME-Zarr well"))?;
    let well = parsed
        .fields
        .well
        .ok_or_else(|| format!("{path}: no well metadata"))?;
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
            let error = format!("{}: the read stopped", self.path);
            let _ = tx.send((std::mem::take(&mut self.path), Err(error)));
        }
    }
}

pub(super) fn fetch_well(url: String, path: String, tx: Sender<WellResult>) {
    let reply = Reply {
        path: path.clone(),
        tx: Some(tx),
    };
    #[cfg(not(target_arch = "wasm32"))]
    crate::utils::executor::TaskExecutor::spawn_background(move || {
        let rt = crate::utils::executor::get_shared_tokio_rt();
        reply.send(rt.block_on(read_well(url, path)));
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        reply.send(read_well(url, path).await);
    });
}
