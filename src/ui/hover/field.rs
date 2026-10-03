//! Structured key/value rows shown in the hover card coordinate grid.

/// One coordinate row of the hover card: a dimension label and its formatted value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverField {
    pub label: String,
    pub value: String,
}

impl HoverField {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    /// 1-based position within a dimension, e.g. `6 / 10`.
    pub fn index_of(label: impl Into<String>, idx: usize, total: usize) -> Self {
        Self::new(label, format!("{} / {}", idx + 1, total))
    }
}
