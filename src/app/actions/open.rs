//! Opening a dataset by name through the calls the pills and the gallery
//! make (`app::sources`); the commands after it wait for it to load, and a
//! gallery entry's view.

use super::resolve::Resolution;
use super::source_names::SourceRef;
use super::text::is_address;
use super::{Done, Failure, NameKind, Outcome, resolved};
use crate::app::OctantApp;

impl OctantApp {
    pub(super) fn open_by_name(&mut self, name: &str) -> Outcome {
        let source = match self.vocabulary().source(name) {
            Resolution::NotFound(_) if is_address(name) => SourceRef::Store(name.trim().into()),
            resolution => resolved(resolution, NameKind::Source, name)?,
        };
        let shown = match &source {
            SourceRef::Sample(uri) => {
                self.open_sample(uri);
                name.to_string()
            }
            SourceRef::Entry(entry) => {
                if self.gallery_entry_refused(entry) {
                    return Err(Failure::Refused);
                }
                self.layout.show_gallery_window = false;
                // Its view (a share link) runs next, as this command: its
                // `OpenSource` waits for the store, and the commands after
                // it wait for the view.
                if let Some(actions) = crate::app::share::parse_share(&entry.link()) {
                    self.commands.run_next(actions);
                    return Ok(Done::Opening(entry.label.clone()));
                }
                self.submit_or_activate_source(&entry.store, None);
                entry.label.clone()
            }
            SourceRef::Store(uri) => {
                self.submit_or_activate_source(uri, None);
                name.to_string()
            }
        };
        self.commands.await_dataset(&self.selected.store_target);
        Ok(Done::Opening(shown))
    }
}
