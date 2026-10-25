//! Opening a dataset by name through the calls the pills and the catalog
//! make (`app::sources`); the commands after it wait for it to load.

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
                if !self.open_catalog_entry(entry) {
                    return Err(Failure::Refused);
                }
                entry.label.to_string()
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
