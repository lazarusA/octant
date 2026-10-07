//! Band names of a composite's channel dimension, from the dataset's labels.

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, VariableInfo};

/// Labels of the channel dimension, when the dataset stores one per band.
pub(super) struct BandNames<'a> {
    dim: &'a str,
    labels: Option<&'a [String]>,
}

impl<'a> BandNames<'a> {
    pub(super) fn resolve(
        app: &OctantApp,
        meta: Option<&'a DatasetMetadata>,
        var: Option<&'a VariableInfo>,
    ) -> Self {
        let c_dim = app.channel_dim_index().unwrap_or(0);
        let dim = var
            .and_then(|v| v.dimension_names.get(c_dim))
            .map_or("band", String::as_str);
        let count = var.and_then(|v| v.shape.get(c_dim)).copied();
        let labels = meta
            .and_then(|m| m.get_dim_coords(var.map(|v| v.name.as_str()), dim))
            .and_then(|c| c.labels())
            .filter(|l| count.is_some_and(|n| l.len() as u64 == n));
        Self { dim, labels }
    }

    /// The stored name of `band`, ignoring generated `Band N` placeholders.
    pub(super) fn real_name(&self, band: usize) -> Option<&'a str> {
        let name = self.labels?.get(band)?.trim();
        let placeholder = name
            .strip_prefix("Band ")
            .is_some_and(|n| n.parse::<usize>().is_ok());
        (!name.is_empty() && !placeholder).then_some(name)
    }

    /// The name shown for `band`: its label, else the dimension name and 1-based index.
    pub(super) fn name(&self, band: usize) -> String {
        match self.labels.and_then(|l| l.get(band)).map(|n| n.trim()) {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => {
                let mut chars = self.dim.chars();
                let first = chars.next().map(|c| c.to_ascii_uppercase());
                format!("{}{} {}", first.unwrap_or('B'), chars.as_str(), band + 1)
            }
        }
    }
}
