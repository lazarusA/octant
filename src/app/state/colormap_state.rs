//! Colormap selection state and user-defined colormaps (the only persisted part).

use super::app_state::OctantApp;
use crate::utils::colormap::{CustomColormapSpec, registry};
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "octant.colormaps";

/// Colormap state beyond the active id (which stays on `OctantApp::active_colormap`).
#[derive(Default)]
pub struct ColormapState {
    /// Samples the active colormap from its end.
    pub reversed: bool,
    /// Uses the smooth twin of a short categorical palette (not persisted).
    pub smooth: bool,
    /// User-defined colormaps, registered in `utils::colormap::registry`.
    pub custom: Vec<CustomColormapSpec>,
    /// Registry generation last uploaded to the GPU atlas.
    pub gpu_generation: u64,
    pub picker: crate::ui::colormap::PickerState,
}

/// Persisted colormap preferences (eframe storage). Only user-defined colormaps
/// are kept; the active colormap and the reversed toggle reset on every launch.
#[derive(Serialize, Deserialize, Default)]
struct ColormapPrefs {
    custom: Vec<CustomColormapSpec>,
}

impl OctantApp {
    /// Restores the user's custom colormaps from eframe storage.
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
    }

    pub fn save_colormap_prefs(&self, storage: &mut dyn eframe::Storage) {
        let prefs = ColormapPrefs {
            custom: self.colormaps.custom.clone(),
        };
        eframe::set_value(storage, STORAGE_KEY, &prefs);
    }

    /// Atlas row actually drawn: the previewed or active colormap, or its smooth
    /// twin when "Smooth" is on and the palette has one.
    pub fn effective_colormap(&self) -> u32 {
        let id = self.preview_colormap.unwrap_or(self.active_colormap);
        if self.colormaps.smooth {
            registry::smooth_variant(id).unwrap_or(id)
        } else {
            id
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::Storage as _;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MemoryStorage(HashMap<String, String>);

    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_string(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }

    #[test]
    fn only_custom_colormaps_survive_a_restart() {
        let mut app = OctantApp::default();
        let spec = CustomColormapSpec {
            name: "persist_test_map".into(),
            colors: "black, white".into(),
            ..Default::default()
        };
        let Ok(custom_id) = app.add_custom_colormap(spec) else {
            panic!("valid spec rejected");
        };
        app.active_colormap = custom_id;
        app.colormaps.reversed = true;

        let mut storage = MemoryStorage::default();
        app.save_colormap_prefs(&mut storage);

        let mut restarted = OctantApp::default();
        restarted.load_colormap_prefs(Some(&storage));
        assert_eq!(restarted.active_colormap, registry::default_id());
        assert!(!restarted.colormaps.reversed);
        assert!(
            restarted
                .colormaps
                .custom
                .iter()
                .any(|s| s.name == "persist_test_map")
        );
        registry::remove_custom("custom:persist_test_map");
    }

    #[test]
    fn older_prefs_with_a_saved_selection_are_ignored() {
        let mut storage = MemoryStorage::default();
        let old = "(active: \"cmocean:thermal\", reversed: true, custom: [(name: \"legacy_test_map\", \
                   colors: \"red, blue\", interpolation: Linear, blend: Oklab, classes: 0)])";
        storage.set_string(STORAGE_KEY, old.into());
        let mut app = OctantApp::default();
        app.load_colormap_prefs(Some(&storage));
        assert_eq!(app.active_colormap, registry::default_id());
        assert!(!app.colormaps.reversed);
        assert!(
            app.colormaps
                .custom
                .iter()
                .any(|s| s.name == "legacy_test_map")
        );
        registry::remove_custom("custom:legacy_test_map");
    }
}

#[cfg(test)]
mod smooth_tests {
    use super::*;

    #[test]
    fn smooth_toggle_switches_to_the_twin_only_when_one_exists() {
        let mut app = OctantApp::default();
        let set1 = registry::find("colorbrewer:Set1").unwrap_or(0);
        app.active_colormap = set1;
        assert_eq!(app.effective_colormap(), set1);
        app.colormaps.smooth = true;
        assert_eq!(
            Some(app.effective_colormap()),
            registry::smooth_variant(set1)
        );
        app.active_colormap = registry::default_id();
        assert_eq!(
            app.effective_colormap(),
            registry::default_id(),
            "continuous maps are unaffected"
        );
        assert!(!OctantApp::default().colormaps.smooth, "smooth starts off");
    }
}
