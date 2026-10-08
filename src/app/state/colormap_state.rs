//! Colormap selection state and user-defined colormaps (the only persisted part).

use super::app_state::OctantApp;
use crate::utils::colormap::{ColormapEntry, CustomColormapSpec, registry};
use serde::{Deserialize, Serialize};

pub(super) const STORAGE_KEY: &str = "octant.colormaps";
/// Stored preferences that failed to parse are copied here (then to `.1`, `.2`,
/// ... for later, different ones), so a format change never destroys them.
pub(super) const UNREADABLE_KEY: &str = "octant.colormaps.unreadable";
const MAX_UNREADABLE_BACKUPS: usize = 8;

/// Colormap state beyond the active id (which lives in the plotted layer's `ColorStyle::colormap`).
#[derive(Default)]
pub struct ColormapState {
    /// Samples the active colormap from its end.
    pub reversed: bool,
    /// Uses the smooth twin of a short categorical palette (not persisted).
    pub smooth: bool,
    /// User-defined colormaps, registered in `utils::colormap::registry`.
    pub custom: Vec<CustomColormapSpec>,
    /// Stored specs that no longer build (e.g. after a color parser change); not
    /// registered, but saved back unchanged.
    pub unloaded: Vec<CustomColormapSpec>,
    /// Raw stored preferences that could not be parsed, backed up on save.
    pub unreadable_prefs: Option<String>,
    pub picker: crate::ui::colormap::PickerState,
    /// Opacity curve editor (not persisted).
    pub alpha: super::alpha_state::AlphaCurveState,
}

/// Persisted colormap preferences (eframe storage). Only user-defined colormaps
/// are kept; the active colormap and the reversed toggle reset on every launch.
#[derive(Deserialize, Default)]
#[serde(default)]
struct ColormapPrefs {
    custom: Vec<CustomColormapSpec>,
}

/// Borrowed form of `ColormapPrefs` for saving without cloning the specs.
#[derive(Serialize)]
struct ColormapPrefsRef<'a> {
    custom: Vec<&'a CustomColormapSpec>,
}

impl OctantApp {
    /// Restores the user's custom colormaps from eframe storage.
    pub fn load_colormap_prefs(&mut self, storage: Option<&dyn eframe::Storage>) {
        let Some(storage) = storage else {
            return;
        };
        let Some(prefs) = eframe::get_value::<ColormapPrefs>(storage, STORAGE_KEY) else {
            if let Some(raw) = storage.get_string(STORAGE_KEY) {
                log::warn!("Unreadable colormap preferences; backed up under `{UNREADABLE_KEY}`");
                self.colormaps.unreadable_prefs = Some(raw);
            }
            return;
        };
        for spec in prefs.custom {
            match spec.to_entry() {
                Ok(entry) => {
                    self.register_custom(spec, entry);
                }
                Err(e) => {
                    log::warn!("Stored custom colormap `{}` does not build: {e}", spec.name);
                    // A map that builds wins over a broken one with the same name.
                    if !self.has_custom_name(&spec.name) {
                        self.colormaps.unloaded.push(spec);
                    }
                }
            }
        }
    }

    pub fn save_colormap_prefs(&self, storage: &mut dyn eframe::Storage) {
        if let Some(raw) = &self.colormaps.unreadable_prefs {
            back_up_unreadable(storage, raw);
        }
        let prefs = ColormapPrefsRef {
            custom: self
                .colormaps
                .custom
                .iter()
                .chain(&self.colormaps.unloaded)
                .collect(),
        };
        eframe::set_value(storage, STORAGE_KEY, &prefs);
    }

    /// Atlas row actually drawn: the previewed or active colormap, or its smooth
    /// twin when "Smooth" is on and the palette has one.
    pub fn effective_colormap(&self) -> u32 {
        self.shown_colormap(
            self.preview_colormap
                .unwrap_or(self.layers.base.color.colormap),
        )
    }

    /// Atlas row drawn for colormap `id`: its smooth twin when "Smooth" is on.
    /// The toggle only shows (and so only applies) while the active map has a twin.
    pub fn shown_colormap(&self, id: u32) -> u32 {
        if self.colormaps.smooth
            && registry::smooth_variant(self.layers.base.color.colormap).is_some()
        {
            registry::smooth_variant(id).unwrap_or(id)
        } else {
            id
        }
    }

    /// Whether a saved custom map (built or not) is named `name` (trimmed).
    pub fn has_custom_name(&self, name: &str) -> bool {
        let name = name.trim();
        let colormaps = &self.colormaps;
        colormaps
            .custom
            .iter()
            .chain(&colormaps.unloaded)
            .any(|s| s.name.trim() == name)
    }

    /// Registers (or replaces) a custom colormap and returns its id.
    pub fn add_custom_colormap(&mut self, spec: CustomColormapSpec) -> Result<u32, String> {
        let entry = spec.to_entry()?;
        Ok(self.register_custom(spec, entry))
    }

    /// Registers `entry` built from `spec`, replacing a map with the same name.
    fn register_custom(&mut self, spec: CustomColormapSpec, entry: ColormapEntry) -> u32 {
        let id = registry::upsert_custom(entry);
        let name = spec.name.trim();
        self.colormaps.unloaded.retain(|s| s.name.trim() != name);
        match self
            .colormaps
            .custom
            .iter_mut()
            .find(|s| s.name.trim() == name)
        {
            Some(existing) => *existing = spec,
            None => self.colormaps.custom.push(spec),
        }
        id
    }

    /// Saves `spec`, replacing the custom map keyed `replaces` when it was
    /// renamed, and returns the saved map's id.
    pub fn save_custom_colormap(
        &mut self,
        spec: CustomColormapSpec,
        replaces: Option<&str>,
    ) -> Result<u32, String> {
        let key = spec.key();
        let id = self.add_custom_colormap(spec)?;
        match replaces.filter(|old| *old != key) {
            // Removing a map shifts the ids after it, so look the new one up again.
            Some(old) => {
                self.remove_custom_colormap(old);
                registry::find(&key).ok_or_else(|| format!("Colormap {key} was not registered"))
            }
            None => Ok(id),
        }
    }

    /// Removes a custom colormap, keeping the active selection on the same map
    /// (or the default when the active map itself was removed).
    pub fn remove_custom_colormap(&mut self, key: &str) {
        let active_key = registry::key_of(self.layers.base.color.colormap);
        registry::remove_custom(key);
        self.colormaps.custom.retain(|s| !s.has_key(key));
        self.colormaps.unloaded.retain(|s| !s.has_key(key));
        self.layers.base.color.colormap =
            registry::find(&active_key).unwrap_or_else(registry::default_id);
        self.preview_colormap = None;
        // Ids after the removed row shifted, so a remembered hover is stale.
        self.colormaps.picker.last_hovered = None;
    }
}

/// Copies unparseable prefs to the first free backup slot, unless a slot
/// already holds them, so older backups are never overwritten.
fn back_up_unreadable(storage: &mut dyn eframe::Storage, raw: &str) {
    for n in 0..MAX_UNREADABLE_BACKUPS {
        let key = match n {
            0 => UNREADABLE_KEY.to_string(),
            n => format!("{UNREADABLE_KEY}.{n}"),
        };
        match storage.get_string(&key) {
            Some(existing) if existing == raw => return,
            Some(_) => {}
            None => {
                storage.set_string(&key, raw.to_owned());
                return;
            }
        }
    }
    log::warn!("All {MAX_UNREADABLE_BACKUPS} colormap preference backups are in use");
}

#[cfg(test)]
#[path = "colormap_state_tests.rs"]
mod tests;
