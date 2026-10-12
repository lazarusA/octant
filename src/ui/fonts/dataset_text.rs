//! A dataset's CJK text, gathered incrementally: its names and attributes
//! once per dataset (`metadata_generation`), then each dimension's
//! coordinate labels once, when they arrive, so the many coordinate
//! arrivals of a session don't re-read everything (large label coordinates
//! such as station names would otherwise be re-scanned each time).

use super::CjkFont;
use super::scan::{Script, Scripts, script_of};
use crate::data::{CoordValues, DatasetMetadata};
use std::collections::{BTreeSet, HashSet};

/// The CJK scripts and characters of one dataset's text.
#[derive(Debug, Default)]
pub struct DatasetText {
    generation: Option<u64>,
    pub scripts: Scripts,
    /// Every CJK character met (`scan::script_of`).
    chars: BTreeSet<char>,
    /// Dimensions whose coordinate labels were read.
    labels_read: HashSet<String>,
}

impl DatasetText {
    /// Everything in `meta`, at once.
    pub fn of(meta: &DatasetMetadata) -> Self {
        let mut text = Self::default();
        text.update(meta, 0);
        text
    }

    /// Reads what is new in `meta` (dataset `generation`): all of it for a
    /// new dataset, else only coordinate labels not read before.
    pub fn update(&mut self, meta: &DatasetMetadata, generation: u64) {
        if self.generation != Some(generation) {
            *self = Self {
                generation: Some(generation),
                ..Self::default()
            };
            self.add(&meta.name);
            for var in &meta.variables {
                self.add(&var.name);
                for text in [&var.long_name, &var.units].into_iter().flatten() {
                    self.add(text);
                }
                for name in &var.dimension_names {
                    self.add(name);
                }
                for (key, value) in &var.attributes {
                    self.add(key);
                    self.add(value);
                }
            }
        }
        for (dim, coords) in &meta.dimension_coordinates {
            let CoordValues::Labels(labels) = coords else {
                continue;
            };
            if self.labels_read.insert(dim.clone()) {
                self.add(dim);
                for label in labels.iter() {
                    self.add(label);
                }
            }
        }
    }

    fn add(&mut self, text: &str) {
        self.scripts.add(text);
        if self.scripts.any() {
            self.chars.extend(
                text.chars()
                    .filter(|&c| !c.is_ascii() && script_of(c).is_some()),
            );
        }
    }

    /// The characters by the font that draws them (`CjkFont::ALL` order):
    /// kana Japanese, Hangul Korean, Han and CJK symbols `han_font`.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn by_font(&self, han_font: CjkFont) -> [BTreeSet<char>; 3] {
        let mut chars: [BTreeSet<char>; 3] = Default::default();
        let slot = |font: CjkFont| CjkFont::ALL.iter().position(|&f| f == font).unwrap_or(0);
        for &c in &self.chars {
            let font = match script_of(c) {
                Some(Script::Kana) => CjkFont::Jp,
                Some(Script::Hangul) => CjkFont::Kr,
                Some(Script::Han | Script::Other) | None => han_font,
            };
            if let Some(set) = chars.get_mut(slot(font)) {
                set.insert(c);
            }
        }
        chars
    }
}
