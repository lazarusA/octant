//! The open high-content screening plate: its layout, the well and
//! acquisition picked, and each well's fields, read when the well is first
//! picked. A field opens as the active dataset (`{plate}/{well}/{field}`)
//! while the plate stays browsable.

#[cfg(test)]
#[path = "plate_tests.rs"]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use super::StoreKind;
use crate::data::ome::{OmePlate, OmeWell};

/// A well's fields, as far as they are known.
#[derive(Debug, Clone, PartialEq)]
pub enum WellFields {
    Loading,
    Ready(Arc<OmeWell>),
    Failed(String),
}

/// The plate being browsed.
pub struct PlateState {
    /// The plate's own store (its root group).
    pub url: String,
    pub store_kind: StoreKind,
    pub plate: Arc<OmePlate>,
    /// Index into `plate.wells` of the picked well.
    pub well: Option<usize>,
    /// The acquisition whose fields are listed (`None`: all).
    pub acquisition: Option<u64>,
    /// The URL of the field shown as the active dataset.
    pub opened: Option<String>,
    wells: HashMap<String, WellFields>,
    tx: Sender<(String, Result<OmeWell, String>)>,
    rx: Receiver<(String, Result<OmeWell, String>)>,
}

impl PlateState {
    pub fn new(url: &str, store_kind: StoreKind, plate: Arc<OmePlate>) -> Self {
        let (tx, rx) = channel();
        Self {
            url: url.trim_end_matches('/').to_string(),
            store_kind,
            plate,
            well: None,
            acquisition: None,
            opened: None,
            wells: HashMap::new(),
            tx,
            rx,
        }
    }

    /// Whether `target` is the plate or one of its fields.
    pub fn contains(&self, target: &str) -> bool {
        let target = target.trim_end_matches('/');
        target == self.url
            || target
                .strip_prefix(self.url.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
    }

    /// The fields of the well at `path`, fetching them the first time.
    pub fn well_fields(&mut self, path: &str) -> &WellFields {
        if !self.wells.contains_key(path) {
            self.wells.insert(path.to_string(), WellFields::Loading);
            fetch_well(self.url.clone(), path.to_string(), self.tx.clone());
        }
        &self.wells[path]
    }

    /// Takes the wells whose fields arrived; whether any did.
    pub fn poll(&mut self) -> bool {
        let mut any = false;
        while let Ok((path, result)) = self.rx.try_recv() {
            let fields = match result {
                Ok(well) => WellFields::Ready(Arc::new(well)),
                Err(e) => WellFields::Failed(e),
            };
            self.wells.insert(path, fields);
            any = true;
        }
        any
    }

    /// Whether a well's fields are still being read.
    pub fn loading(&self) -> bool {
        self.wells.values().any(|w| *w == WellFields::Loading)
    }

    /// The URL of field `field` of the well at `well`.
    pub fn field_url(&self, well: &str, field: &str) -> String {
        format!(
            "{}/{}/{}",
            self.url,
            well.trim_matches('/'),
            field.trim_matches('/')
        )
    }
}

impl crate::app::OctantApp {
    /// Keeps the plate browsable while `metadata` (just opened) is the plate
    /// or one of its fields; opens a plate it holds; forgets the plate
    /// otherwise. Whether `metadata` is a plate.
    pub(crate) fn follow_plate(&mut self, metadata: &crate::data::DatasetMetadata) -> bool {
        let target = self.selected.store_target.trim_end_matches('/').to_string();
        if let Some(plate) = metadata.ome.as_ref().and_then(|o| o.plate.clone()) {
            self.plate = Some(PlateState::new(&target, self.selected.store_kind, plate));
            self.layout.show_dataset_panel = true;
            return true;
        }
        if !self.plate.as_ref().is_some_and(|p| p.contains(&target)) {
            self.plate = None;
        }
        false
    }

    /// Opens field `field` of the plate's well `well` as the active dataset.
    pub(crate) fn open_plate_field(&mut self, well: &str, field: &str) {
        let Some(plate) = &mut self.plate else {
            return;
        };
        let url = plate.field_url(well, field);
        let kind = plate.store_kind;
        plate.opened = Some(url.clone());
        self.submit_or_activate_source(&url, Some(kind));
    }
}

/// The well group's metadata: `zarr.json` (Zarr v3) or `.zattrs` (Zarr v2).
async fn well_attributes(
    url: &str,
    path: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let mut last_error = String::new();
    for name in ["zarr.json", ".zattrs"] {
        let key = format!("{url}/{path}/{name}");
        match read_bytes(&key).await {
            Ok(bytes) => {
                let mut doc: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if name == "zarr.json" {
                    return match doc.remove("attributes") {
                        Some(serde_json::Value::Object(attrs)) => Ok(attrs),
                        _ => Ok(serde_json::Map::new()),
                    };
                }
                return Ok(doc);
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
async fn read_well(url: String, path: String) -> Result<OmeWell, String> {
    let attrs = well_attributes(&url, &path).await?;
    let parsed = crate::data::ome::parse::parse_group(&attrs, &|_| None)
        .ok_or_else(|| format!("{path}: not an OME-Zarr well"))?;
    let well = parsed
        .fields
        .well
        .ok_or_else(|| format!("{path}: no well metadata"))?;
    Ok(crate::data::ome::plate::well_info(&well))
}

fn fetch_well(url: String, path: String, tx: Sender<(String, Result<OmeWell, String>)>) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::utils::executor::TaskExecutor::spawn_background(move || {
        let rt = crate::utils::executor::get_shared_tokio_rt();
        let result = rt.block_on(read_well(url, path.clone()));
        let _ = tx.send((path, result));
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let result = read_well(url, path.clone()).await;
        let _ = tx.send((path, result));
    });
}
