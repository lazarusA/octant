//! The open high-content screening plate: its layout, the well and
//! acquisition picked, and each well's fields, read when the well is first
//! picked. A field opens as the active dataset (`{plate}/{well}/{field}`)
//! while the plate stays browsable.

#[path = "plate_fetch.rs"]
mod fetch;
#[cfg(test)]
#[path = "plate_tests.rs"]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use fetch::{WellResult, ZarrV2, fetch_well};

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
    /// Whether the wells answered as Zarr v2, once one did.
    zarr_v2: ZarrV2,
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
            zarr_v2: ZarrV2::default(),
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
            fetch_well(
                self.url.clone(),
                path.to_string(),
                self.zarr_v2.clone(),
                self.tx.clone(),
            );
        }
        &self.wells[path]
    }

    /// The fields of the well at `path`, as if read.
    #[cfg(test)]
    pub fn set_well_fields(&mut self, path: &str, fields: WellFields) {
        self.wells.insert(path.to_string(), fields);
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

    /// Takes the plate's arrived wells, repainting while any is still read.
    pub(crate) fn poll_plate(&mut self, ctx: &egui::Context) {
        let Some(plate) = &mut self.plate else {
            return;
        };
        if plate.poll() {
            ctx.request_repaint();
        } else if plate.loading() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
