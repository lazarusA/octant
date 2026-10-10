//! Where a line plot's lines live in the base layer's data: one `LineLayout`
//! read by the GPU payload, the hover and the series colorbar, so they agree
//! on which line is which without building the payload.

use super::app_state::OctantApp;

/// `line_count` lines of `profile_length` samples: sample `s` of line `l` is
/// `values[first + l * line_stride + s * sample_stride]` (NaN past the end).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineLayout {
    pub profile_length: usize,
    pub line_count: usize,
    first: usize,
    line_stride: usize,
    sample_stride: usize,
}

impl LineLayout {
    /// Every line of `count` (All Lines Series), or only line `pick` of them.
    fn lines(len: usize, count: usize, strides: (usize, usize), pick: Option<usize>) -> Self {
        let (line_stride, sample_stride) = strides;
        let (first, line_count) = match pick {
            Some(line) => (line.min(count.saturating_sub(1)) * line_stride, 1),
            None => (0, count),
        };
        Self {
            profile_length: len,
            line_count: if count == 0 { 0 } else { line_count },
            first,
            line_stride,
            sample_stride,
        }
    }

    /// Sample `sample` of line `line` in `values`.
    pub fn value(&self, values: &[f32], line: usize, sample: usize) -> f32 {
        let idx = self.first + line * self.line_stride + sample * self.sample_stride;
        values.get(idx).copied().unwrap_or(f32::NAN)
    }
}

/// What the line renderer draws: each line with data as its line index (the
/// bits of a `u32`) followed by its `profile_length` samples.
pub struct LinePayload {
    pub values: Vec<f32>,
    pub profile_length: u32,
    /// Lines in `values`.
    pub drawn_lines: u32,
    /// Lines in the plot, with or without data (`LineLayout::line_count`).
    pub line_count: u32,
}

impl OctantApp {
    /// The base layer's line data and how its lines lie in it.
    pub fn line_layout(&self) -> (&[f32], LineLayout) {
        let data = &self.layers.base.data;
        let pick = (!self.line_plot_all_series).then_some(self.line_profile_slice_idx);
        if self.line_profile_dim_idx == 2
            && let Some(v) = data.volume.as_ref().filter(|v| v.depth > 1)
        {
            let pixels = v.width * v.height;
            let layout = LineLayout::lines(v.depth, pixels, (1, pixels), pick);
            (&v.values, layout)
        } else if let Some(m) = &data.matrix {
            let layout = match self.line_profile_dim_idx {
                0 => LineLayout::lines(m.width, m.height, (m.width, 1), pick),
                _ => LineLayout::lines(m.height, m.width, (1, m.width), pick),
            };
            (&m.values, layout)
        } else {
            (&[], LineLayout::default())
        }
    }

    /// The renderer's payload: lines without a finite value are left out.
    pub fn line_payload(&self) -> LinePayload {
        let (values, layout) = self.line_layout();
        let len = layout.profile_length;
        let mut payload = Vec::with_capacity(layout.line_count * (len + 1));
        let mut drawn = 0;
        for line in 0..layout.line_count {
            let sample = |s: usize| layout.value(values, line, s);
            if !(0..len).any(|s| sample(s).is_finite()) {
                continue;
            }
            payload.push(f32::from_bits(line as u32));
            payload.extend((0..len).map(sample));
            drawn += 1;
        }
        LinePayload {
            values: payload,
            profile_length: len as u32,
            drawn_lines: drawn,
            line_count: layout.line_count as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineLayout;

    const VALUES: [f32; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    #[test]
    fn rows_and_columns_of_a_matrix() {
        // 3 wide, 2 high.
        let rows = LineLayout::lines(3, 2, (3, 1), None);
        assert_eq!(rows.line_count, 2);
        assert_eq!(rows.value(&VALUES, 1, 2), 6.0);
        let cols = LineLayout::lines(2, 3, (1, 3), None);
        assert_eq!(cols.value(&VALUES, 2, 1), 6.0);
        assert_eq!(cols.value(&VALUES, 0, 1), 4.0);
    }

    #[test]
    fn a_picked_line_is_clamped_and_short_data_reads_nan() {
        let row = LineLayout::lines(3, 2, (3, 1), Some(9));
        assert_eq!((row.line_count, row.value(&VALUES, 0, 0)), (1, 4.0));
        assert!(row.value(&VALUES[..4], 0, 2).is_nan());
        assert_eq!(LineLayout::lines(3, 0, (3, 1), Some(0)).line_count, 0);
    }
}
