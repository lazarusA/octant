//! Typed dimension coordinates: evenly spaced axes in constant space, uneven numbers, text
//! labels, or only the endpoints. Cloning shares the values.

use std::sync::Arc;

/// The coordinate values along one dimension.
#[derive(Debug, Clone, PartialEq)]
pub enum CoordValues {
    /// `start + i * step` for `i < len`.
    Regular { start: f64, step: f64, len: usize },
    /// One number per index, unevenly spaced.
    Values(Arc<[f64]>),
    /// One text label per index.
    Labels(Arc<[String]>),
    /// Only the first and last of `len` values are known; others are interpolated.
    Endpoints { first: f64, last: f64, len: usize },
}

/// One coordinate: a number or a text label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordValue<'a> {
    Number(f64),
    Label(&'a str),
}

impl CoordValues {
    /// Numbers read from storage: evenly spaced ones are kept as [`CoordValues::Regular`].
    /// `f32_source` widens the spacing tolerance to `f32` rounding. `None` when empty.
    pub fn from_values(values: Vec<f64>, f32_source: bool) -> Option<Self> {
        let mut check = SpacingCheck::new(*values.first()?, values.len(), *values.last()?);
        check.f32_source = f32_source;
        if values.iter().enumerate().all(|(i, &v)| check.fits(i, v)) {
            Some(check.regular())
        } else {
            Some(Self::Values(values.into()))
        }
    }

    /// Text labels; `None` when empty.
    pub fn from_labels(labels: Vec<String>) -> Option<Self> {
        (!labels.is_empty()).then(|| Self::Labels(labels.into()))
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Regular { len, .. } | Self::Endpoints { len, .. } => *len,
            Self::Values(v) => v.len(),
            Self::Labels(l) => l.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether every index holds its stored value rather than an interpolated one.
    pub fn is_exact(&self) -> bool {
        !matches!(self, Self::Endpoints { .. })
    }

    /// Whether these values describe a dimension of `dim_len` indices one to one.
    pub fn matches(&self, dim_len: usize) -> bool {
        self.is_exact() && self.len() == dim_len
    }

    /// The coordinate at index `i`.
    pub fn get(&self, i: usize) -> Option<CoordValue<'_>> {
        match self {
            Self::Labels(l) => l.get(i).map(|s| CoordValue::Label(s.as_str())),
            _ => self.number(i).map(CoordValue::Number),
        }
    }

    /// The number at index `i`; `None` for labels and out-of-range indices.
    pub fn number(&self, i: usize) -> Option<f64> {
        match self {
            Self::Regular { start, step, len } => (i < *len).then(|| start + i as f64 * step),
            Self::Values(v) => v.get(i).copied(),
            Self::Labels(_) => None,
            Self::Endpoints { first, last, len } => {
                (i < *len).then(|| interpolate(*first, *last, i, *len))
            }
        }
    }

    /// The label at index `i`; `None` for numbers.
    pub fn label(&self, i: usize) -> Option<&str> {
        match self {
            Self::Labels(l) => l.get(i).map(String::as_str),
            _ => None,
        }
    }

    pub fn labels(&self) -> Option<&[String]> {
        match self {
            Self::Labels(l) => Some(l),
            _ => None,
        }
    }

    pub fn first_number(&self) -> Option<f64> {
        self.number(0)
    }

    pub fn last_number(&self) -> Option<f64> {
        self.number(self.len().checked_sub(1)?)
    }

    /// The number at index `i` of a dimension of `dim_len` indices: the stored value when
    /// the values match the dimension, else interpolated between the first and last.
    pub fn number_for(&self, i: usize, dim_len: usize) -> Option<f64> {
        if self.matches(dim_len) {
            return self.number(i);
        }
        let (first, last) = (self.first_number()?, self.last_number()?);
        Some(interpolate(
            first,
            last,
            i.min(dim_len.saturating_sub(1)),
            dim_len,
        ))
    }

    /// `(min, max)` of the numbers at indices `start` and `end` of a `dim_len` dimension.
    pub fn range_bounds(&self, start: usize, end: usize, dim_len: usize) -> Option<(f64, f64)> {
        let a = self.number_for(start, dim_len)?;
        let b = self.number_for(end, dim_len)?;
        Some((a.min(b), a.max(b)))
    }
}

fn interpolate(first: f64, last: f64, i: usize, len: usize) -> f64 {
    if len > 1 {
        first + (last - first) * (i as f64 / (len - 1) as f64)
    } else {
        first
    }
}

/// Checks values one at a time against an even spacing from `first` to `last`, so readers
/// can test a coordinate chunk by chunk without keeping it.
#[derive(Debug, Clone, Copy)]
pub struct SpacingCheck {
    pub start: f64,
    pub step: f64,
    pub len: usize,
    /// Allow the rounding of values stored as `f32`.
    pub f32_source: bool,
    /// Largest magnitude of the endpoints: `f32` rounding of `first` and `last` shifts
    /// every expected value by up to this scale's rounding, even where values are near 0.
    scale: f64,
}

/// Spacing tolerance, as a fraction of the step.
const STEP_TOLERANCE: f64 = 1e-6;

impl SpacingCheck {
    pub fn new(first: f64, len: usize, last: f64) -> Self {
        let step = if len > 1 {
            (last - first) / (len - 1) as f64
        } else {
            0.0
        };
        Self {
            start: first,
            step,
            len,
            f32_source: false,
            scale: first.abs().max(last.abs()),
        }
    }

    /// The value index `i` takes on the even spacing.
    pub fn expected(&self, i: usize) -> f64 {
        self.start + i as f64 * self.step
    }

    /// Whether `value` at index `i` lies on the even spacing.
    pub fn fits(&self, i: usize, value: f64) -> bool {
        let expected = self.expected(i);
        let mut tolerance = self.step.abs() * STEP_TOLERANCE;
        if self.f32_source {
            let magnitude = self.scale.max(value.abs());
            tolerance += magnitude * f64::from(f32::EPSILON) * 2.0;
        }
        (value - expected).abs() <= tolerance
    }

    pub fn regular(&self) -> CoordValues {
        CoordValues::Regular {
            start: self.start,
            step: self.step,
            len: self.len,
        }
    }
}
