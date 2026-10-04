//! Colormap selection, user-defined colormaps and their persistence.

use super::app_state::OctantApp;
use crate::utils::colormap::{CustomColormapSpec, registry};
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "octant.colormaps";

/// Colormap state beyond the active id (which stays on `OctantApp::active_colormap`).
#[derive(Default)]
pub struct ColormapState {
    /// Samples the active colormap from its end.
    pub reversed: bool,
    /// User-defined colormaps, registered in `utils::colormap::registry`.
    pub custom: Vec<CustomColormapSpec>,
    /// Registry generation last uploaded to the GPU atlas.
    pub gpu_generation: u64,
    pub picker: crate::ui::colormap::PickerState,
}

/// Persisted colormap preferences (eframe storage).
#[derive(Serialize, Deserialize, Default)]
struct ColormapPrefs {
    active: String,
    reversed: bool,
    custom: Vec<CustomColormapSpec>,
}

impl OctantApp {
    /// Restores custom colormaps and the active selection from eframe storage.
    pub fn load_colormap_prefs(&mut self, storage: Option<&dyn eframe::Storage>) {
        let Some(prefs) = storage.and_then(|s| eframe::get_value::<ColormapPrefs>(s, STORAGE_KEY))
        else {
            return;
        };
        for spec in prefs.custom {
            if let Err(e) = self.add_custom_colormap(spec) {
                log::warn!("Skipping stored custom colormap: {e}");
            }
        }
        self.colormaps.reversed = prefs.reversed;
        self.active_colormap = registry::find(&prefs.active).unwrap_or_else(registry::default_id);
    }

    pub fn save_colormap_prefs(&self, storage: &mut dyn eframe::Storage) {
        let prefs = ColormapPrefs {
            active: registry::key_of(self.active_colormap),
            reversed: self.colormaps.reversed,
            custom: self.colormaps.custom.clone(),
        };
        eframe::set_value(storage, STORAGE_KEY, &prefs);
    }

    /// Registers (or replaces) a custom colormap and returns its id.
    pub fn add_custom_colormap(&mut self, spec: CustomColormapSpec) -> Result<u32, String> {
        let entry = spec.to_entry()?;
        let id = registry::upsert_custom(entry);
        let key = spec.key();
        match self.colormaps.custom.iter_mut().find(|s| s.key() == key) {
            Some(existing) => *existing = spec,
            None => self.colormaps.custom.push(spec),
        }
        Ok(id)
    }

    /// Removes a custom colormap, keeping the active selection on the same map
    /// (or the default when the active map itself was removed).
    pub fn remove_custom_colormap(&mut self, key: &str) {
        let active_key = registry::key_of(self.active_colormap);
        registry::remove_custom(key);
        self.colormaps.custom.retain(|s| s.key() != key);
        self.active_colormap = registry::find(&active_key).unwrap_or_else(registry::default_id);
        self.preview_colormap = None;
    }
}
