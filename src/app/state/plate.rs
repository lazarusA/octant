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
    /// Unreadable (reported once); picking the well again retries.
    Failed,
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
    /// The field shown as the active dataset: its well's index into
    /// `plate.wells` and its path in the well.
    pub opened: Option<(usize, String)>,
    wells: HashMap<String, WellFields>,
    tx: Sender<WellResult>,
    rx: Receiver<WellResult>,
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

    /// Reads the fields of the well at `path` again after a failure.
    pub fn retry_well(&mut self, path: &str) {
        if self.wells.get(path) == Some(&WellFields::Failed) {
            self.wells.remove(path);
        }
    }

    /// Takes the wells whose fields arrived; whether any did.
    pub fn poll(&mut self) -> bool {
        let mut any = false;
        while let Ok((path, result)) = self.rx.try_recv() {
            let fields = match result {
                Ok(well) => WellFields::Ready(Arc::new(well)),
                Err(e) => {
                    crate::ui::toast::report(
                        crate::ui::toast::Severity::Warning,
                        crate::i18n::tr(crate::i18n::NoticesKey::CouldntLoadData),
                        e,
                    );
                    WellFields::Failed
                }
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

    /// The well index and field path of `target`, a field of this plate.
    fn field_of(&self, target: &str) -> Option<(usize, String)> {
        let rest = target.strip_prefix(self.url.as_str())?.trim_matches('/');
        self.plate.wells.iter().enumerate().find_map(|(i, w)| {
            let field = rest.strip_prefix(w.path.trim_matches('/'))?;
            let field = field.strip_prefix('/')?.trim_matches('/');
            (!field.is_empty()).then(|| (i, field.to_string()))
        })
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
            match &mut self.plate {
                // The same plate again keeps the picked well and read fields.
                Some(open) if open.url == target => {
                    open.plate = plate;
                    open.opened = None;
                }
                _ => self.plate = Some(PlateState::new(&target, self.selected.store_kind, plate)),
            }
            self.layout.show_dataset_panel = true;
            return true;
        }
        match &mut self.plate {
            Some(plate) if plate.contains(&target) => {
                // Marked once the field has opened, however it was opened.
                plate.opened = plate.field_of(&target);
                if let Some((well, _)) = plate.opened {
                    plate.well = Some(well);
                }
            }
            _ => self.plate = None,
        }
        false
    }

    /// Opens field `field` of the plate's well at index `well` as the active
    /// dataset.
    pub(crate) fn open_plate_field(&mut self, well: usize, field: &str) {
        let Some(plate) = &mut self.plate else {
            return;
        };
        let Some(path) = plate.plate.wells.get(well).map(|w| w.path.as_str()) else {
            return;
        };
        let url = plate.field_url(path, field);
        let kind = plate.store_kind;
        self.submit_or_activate_source(&url, Some(kind));
    }
}

/// The well group's attributes: `zarr.json` (Zarr v3) or `.zattrs` (Zarr v2).
async fn well_attributes(
    url: &str,
    path: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let mut last_error = String::new();
    for name in ["zarr.json", ".zattrs"] {
        let key = format!("{url}/{path}/{name}");
        match read_bytes(&key).await {
            Ok(bytes) => {
                return crate::data::ome::reader::attributes_of(&bytes)
                    .map(|(attrs, _)| attrs)
                    .ok_or_else(|| format!("{key}: not a JSON object"));
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

type WellResult = (String, Result<OmeWell, String>);

/// Sends a well's result once; a read that ends without one (a panic)
/// sends a failure, so the well never stays loading.
struct Reply {
    path: String,
    tx: Option<Sender<WellResult>>,
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

fn fetch_well(url: String, path: String, tx: Sender<WellResult>) {
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
