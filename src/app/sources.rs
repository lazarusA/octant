//! Opening a dataset as each entry point offers one: a landing-screen
//! sample, a gallery entry, any address. The pills, the gallery and the
//! action layer all open through here.

use super::OctantApp;
use crate::gallery::GalleryEntry;
use crate::i18n::{DatasetKey, tr};
use crate::ui::toast::Severity;

/// The landing sample that opens the gallery rather than a store.
pub const GALLERY_SAMPLE: &str = "gallery";

impl OctantApp {
    /// Opens a landing sample (`ui::hero::chips::SAMPLES`): the gallery, or
    /// its store, shown in the intake field.
    pub fn open_sample(&mut self, uri: &str) {
        if uri == GALLERY_SAMPLE {
            self.layout.show_gallery_window = true;
            return;
        }
        self.layout.hero_state.input = uri.to_string();
        self.submit_or_activate_source(uri, None);
    }

    /// Opens gallery `entry` and closes the gallery; false, with a notice,
    /// when the browser cannot read its server (no CORS headers).
    pub fn open_gallery_entry(&mut self, entry: &GalleryEntry) -> bool {
        if self.gallery_entry_refused(entry) {
            return false;
        }
        self.submit_or_activate_source(&entry.link(), None);
        self.layout.show_gallery_window = false;
        true
    }

    /// Whether the browser cannot read `entry`'s server (no CORS headers),
    /// saying so.
    pub(crate) fn gallery_entry_refused(&mut self, entry: &GalleryEntry) -> bool {
        let refused = cfg!(target_arch = "wasm32") && entry.desktop_only;
        if refused {
            self.notify(
                Severity::Warning,
                tr(DatasetKey::BadgeDesktopOnly),
                tr(DatasetKey::DesktopOnlyHint),
            );
        }
        refused
    }
}
