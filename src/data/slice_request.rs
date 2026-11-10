//! Generic N-dimensional selections used to describe a block load.

use zarrs::array::ArraySubset;

use crate::data::blocks::BlockStoreError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

    /// `[start, end)` within a dimension of `len`: the start moved inside it and at least
    /// one element, so an inverted range or one past the end reads one; none when `len`
    /// is 0.
    pub fn clamp(&self, len: usize) -> (usize, usize) {
        let (start, end) = self.bounds();
        let start = start.min(len.saturating_sub(1));
        (start, end.max(start.saturating_add(1)).min(len))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

    /// The `(origin, shape)` window this request reads from an array of `shape`, each
    /// dimension clamped by [`DimensionSelection::clamp`]: the one window of every reader,
    /// and of what the browser preloads. An error when the ranks differ.
    pub fn window(&self, shape: &[u64]) -> Result<(Vec<usize>, Vec<usize>), BlockStoreError> {
        if self.selections.len() != shape.len() {
            return Err(format!(
                "fetch_block: selection has {} dimension(s) but '{}' has rank {}",
                self.selections.len(),
                self.variable,
                shape.len()
            )
            .into());
        }
        Ok(self
            .selections
            .iter()
            .zip(shape)
            .map(|(sel, &len)| {
                let (start, end) = sel.clamp(usize::try_from(len).unwrap_or(usize::MAX));
                (start, end - start)
            })
            .unzip())
    }

    /// Selection `i` clamped to `len` ([`DimensionSelection::clamp`]), the whole
    /// dimension when the request has no selection `i`.
    pub fn clamped(&self, i: usize, len: usize) -> (usize, usize) {
        self.selections.get(i).map_or((0, len), |s| s.clamp(len))
    }

    /// [`Self::window`] as a Zarr `ArraySubset`.
    pub fn to_array_subset(&self, shape: &[u64]) -> Result<ArraySubset, BlockStoreError> {
        let (origin, shape) = self.window(shape)?;
        Ok(window_subset(&origin, &shape))
    }
}

/// The window `origin..origin + shape` as an array subset.
pub fn window_subset(origin: &[usize], shape: &[usize]) -> ArraySubset {
    let ranges: Vec<_> = origin
        .iter()
        .zip(shape)
        .map(|(&start, &len)| start as u64..(start + len) as u64)
        .collect();
    ArraySubset::new_with_ranges(&ranges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_selection_bounds() {
        assert_eq!(DimensionSelection::index(5).bounds(), (5, 6));
        assert_eq!(DimensionSelection::range(2, 10).bounds(), (2, 10));
    }

    /// Every reader's window: past the end, inverted, empty dimensions, rank mismatch
    /// (B10: the browser preloaded another window than it decoded).
    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn windows_clamp_alike() {
        use DimensionSelection as S;
        let window = |sels: Vec<S>, shape: &[u64]| {
            let req = SliceRequest::new("v", sels);
            let subset = req.to_array_subset(shape).map(|s| s.to_ranges());
            let ranges = req.window(shape).map(|(o, s)| {
                o.iter()
                    .zip(&s)
                    .map(|(&o, &s)| o as u64..(o + s) as u64)
                    .collect::<Vec<_>>()
            });
            assert_eq!(
                subset.as_ref().ok(),
                ranges.as_ref().ok(),
                "preload = decode"
            );
            ranges.ok()
        };
        assert_eq!(
            window(vec![S::range(0, 100), S::index(5)], &[20, 10]),
            Some(vec![0..20, 5..6])
        );
        assert_eq!(
            window(vec![S::range(5, 3)], &[10]),
            Some(vec![5..6]),
            "inverted: one"
        );
        assert_eq!(
            window(vec![S::range(30, 40)], &[10]),
            Some(vec![9..10]),
            "past the end"
        );
        assert_eq!(
            window(vec![S::range(0, 10)], &[0]),
            Some(vec![0..0]),
            "empty dimension"
        );
        assert_eq!(
            window(vec![S::range(0, 10)], &[10, 8]),
            None,
            "rank mismatch"
        );
    }
}
