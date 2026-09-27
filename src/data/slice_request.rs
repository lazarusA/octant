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

    /// Computes the total number of elements requested across all dimensions.
    pub fn estimated_elements(&self) -> usize {
        self.selections.iter().fold(1usize, |acc, sel| {
            let (start, end) = sel.bounds();
            acc.saturating_mul(end.saturating_sub(start).max(1))
        })
    }

    /// Converts the slice selections into a Zarr `ArraySubset` clamped against dimension lengths.
    pub fn to_array_subset(&self, shape: &[u64]) -> zarrs::array::ArraySubset {
        let mut ranges = Vec::with_capacity(self.selections.len());
        for (i, sel) in self.selections.iter().enumerate() {
            let Some(&raw_len) = shape.get(i) else {
                continue;
            };
            let dim_len = raw_len as usize;
            if dim_len == 0 {
                ranges.push(0..0);
                continue;
            }
            let (start, end) = sel.bounds();
            let clamped_start = start.min(dim_len.saturating_sub(1));
            let clamped_end = end.clamp(clamped_start, dim_len);
            ranges.push(clamped_start as u64..clamped_end as u64);
        }
        zarrs::array::ArraySubset::new_with_ranges(&ranges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_selection_bounds() {
        assert_eq!(DimensionSelection::index(5).bounds(), (5, 6));
        assert_eq!(DimensionSelection::range(2, 10).bounds(), (2, 10));
    }

    #[test]
    fn test_slice_request_estimated_elements() {
        let req = SliceRequest::new(
            "temperature",
            vec![
                DimensionSelection::range(0, 5),
                DimensionSelection::range(10, 20),
            ],
        );
        assert_eq!(req.estimated_elements(), 5 * 10);
    }

    #[test]
    fn test_slice_request_to_array_subset_clamping() {
        let req = SliceRequest::new(
            "var",
            vec![
                DimensionSelection::range(0, 100),
                DimensionSelection::index(5),
                DimensionSelection::range(50, 60),
            ],
        );
        // Dim 0 length is 20, Dim 1 length is 10, Dim 2 missing
        let shape = vec![20u64, 10u64];
        let subset = req.to_array_subset(&shape);
        assert_eq!(subset.to_ranges(), &[0..20, 5..6]);
    }

    #[test]
    fn test_slice_request_to_array_subset_zero_length() {
        let req = SliceRequest::new("var", vec![DimensionSelection::range(0, 10)]);
        let shape = vec![0u64];
        let subset = req.to_array_subset(&shape);
        assert_eq!(subset.to_ranges(), vec![0..0]);
    }
}
