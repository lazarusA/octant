//! Generic N-dimensional selections used to describe a block load.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DimensionSelection {
    /// Load exactly one element from this dimension.
    Index(usize),

    /// Load `[start, end)`.
    Range { start: usize, end: usize },
}

impl DimensionSelection {
    pub fn index(index: usize) -> Self {
        Self::Index(index)
    }

    pub fn range(start: usize, end: usize) -> Self {
        Self::Range { start, end }
    }

    pub fn bounds(&self) -> (usize, usize) {
        match self {
            Self::Index(index) => (*index, index.saturating_add(1)),
            Self::Range { start, end } => (*start, *end),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SliceRequest {
    pub variable: String,
    pub selections: Vec<DimensionSelection>,
}

impl SliceRequest {
    pub fn new(variable: impl Into<String>, selections: Vec<DimensionSelection>) -> Self {
        Self {
            variable: variable.into(),
            selections,
        }
    }

    pub fn rank(&self) -> usize {
        self.selections.len()
    }

    pub fn index(variable: impl Into<String>, indices: Vec<usize>) -> Self {
        Self {
            variable: variable.into(),
            selections: indices.into_iter().map(DimensionSelection::Index).collect(),
        }
    }

    pub fn full_range(variable: impl Into<String>, shape: &[usize]) -> Self {
        Self {
            variable: variable.into(),
            selections: shape
                .iter()
                .map(|&size| DimensionSelection::Range {
                    start: 0,
                    end: size,
                })
                .collect(),
        }
    }
}
