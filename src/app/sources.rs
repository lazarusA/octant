//! Opening a dataset as each entry point offers one: a landing-screen
//! sample, a catalog entry, any address. The pills, the catalog and the
//! action layer all open through here.

use super::OctantApp;
use crate::catalog::CatalogEntry;
use crate::i18n::{DatasetKey, tr};
use crate::ui::toast::Severity;

/// The landing sample that opens the catalog rather than a store.
pub const CATALOG_SAMPLE: &str = "catalog";

impl OctantApp {
    /// Opens a landing sample (`ui::hero::chips::SAMPLES`): the catalog, or
    /// its store, shown in the intake field.
    pub fn open_sample(&mut self, uri: &str) {
        if uri == CATALOG_SAMPLE {
            self.layout.show_catalog_window = true;
            return;
        }
        self.layout.hero_state.input = uri.to_string();
        self.submit_or_activate_source(uri, None);
    }

    /// Opens catalog `entry` and closes the catalog; false, with a notice,
    /// when the browser cannot read its server (no CORS headers).
    pub fn open_catalog_entry(&mut self, entry: &CatalogEntry) -> bool {
        if cfg!(target_arch = "wasm32") && entry.desktop_only() {
            self.notify(
                Severity::Warning,
                tr(DatasetKey::BadgeDesktopOnly),
                tr(DatasetKey::DesktopOnlyHint),
            );
            return false;
        }
        self.submit_or_activate_source(entry.store.trim(), Some(entry.store_kind));
        self.layout.show_catalog_window = false;
        true
    }
}
