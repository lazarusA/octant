//! Band names of a composite's channel dimension, from the dataset's labels, and the
//! colors they name.

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

const VISIBLE: [&str; 3] = ["red", "green", "blue"];
/// Words naming bands outside the visible range.
const NON_VISIBLE_WORDS: [&str; 6] = ["nir", "swir", "tir", "infrared", "thermal", "rededge"];

/// The words of a band name: runs of ASCII letters and digits.
fn words(name: &str) -> impl Iterator<Item = &str> {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
}

/// Whether a band name names a band outside the visible range (`NIR`, `SWIR 1`,
/// `Near infrared`, `Red edge`, ...).
pub(super) fn is_non_visible(name: &str) -> bool {
    let mut previous = "";
    words(name).any(|w| {
        let red_edge = previous.eq_ignore_ascii_case("red") && w.eq_ignore_ascii_case("edge");
        previous = w;
        red_edge || NON_VISIBLE_WORDS.iter().any(|n| w.eq_ignore_ascii_case(n))
    })
}

/// The visible color (0 red, 1 green, 2 blue) a band name names as a word (`Red`,
/// `Red (B4)`, `B2 blue`); `None` for other names and non-visible bands like `Red edge`.
pub(super) fn visible_color(name: &str) -> Option<usize> {
    if is_non_visible(name) {
        return None;
    }
    words(name).find_map(|w| VISIBLE.iter().position(|c| w.eq_ignore_ascii_case(c)))
}
