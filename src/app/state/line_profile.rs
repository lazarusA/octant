//! Where a line plot's lines live in the base layer's data: one `LineLayout`
//! read by the GPU payload, the hover and the series colorbar, so they agree
//! on which line is which without building the payload. The payload is built
//! and uploaded only when its key (`line_payload_key`) changes.

use std::hash::{DefaultHasher, Hash, Hasher};

use super::app_state::OctantApp;
use crate::plots::{LineRenderer, LineShape};

/// `line_count` lines of `profile_length` samples: sample `s` of line `l` is
/// `values[first + l * line_stride + s * sample_stride]` (NaN past the end).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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
            // An index past any data (on overflow) reads NaN.
            Some(line) => {
                let first = line.min(count.saturating_sub(1)).checked_mul(line_stride);
                (first.unwrap_or(usize::MAX), 1)
            }
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
        let idx = line
            .checked_mul(self.line_stride)
            .zip(sample.checked_mul(self.sample_stride))
            .and_then(|(l, s)| self.first.checked_add(l)?.checked_add(s));
        idx.and_then(|i| values.get(i)).copied().unwrap_or(f32::NAN)
    }

    /// The samples of line `line` in `values`, in order.
    pub fn row<'v>(&self, values: &'v [f32], line: usize) -> impl Iterator<Item = f32> + 'v {
        let start = line.checked_mul(self.line_stride);
        let start = start.and_then(|l| self.first.checked_add(l));
        strided(values, start, self.sample_stride, self.profile_length)
    }

    /// Sample `sample` of every line in `values`, in line order.
    pub fn column<'v>(&self, values: &'v [f32], sample: usize) -> impl Iterator<Item = f32> + 'v {
        let start = sample.checked_mul(self.sample_stride);
        let start = start.and_then(|s| self.first.checked_add(s));
        strided(values, start, self.line_stride, self.line_count)
    }

    /// Upper bound of the payload's bytes: every line drawn.
    fn max_payload_bytes(&self) -> Option<u64> {
        let words = self
            .line_count
            .checked_mul(self.profile_length.checked_add(1)?)?;
        u64::try_from(words).ok()?.checked_mul(4)
    }
}

/// `count` values of `values` from `start`, `step` apart, NaN past the end.
fn strided(
    values: &[f32],
    start: Option<usize>,
    step: usize,
    count: usize,
) -> impl Iterator<Item = f32> + '_ {
    let tail = start.and_then(|s| values.get(s..)).unwrap_or_default();
    let present = tail.iter().step_by(step.max(1)).copied();
    present.chain(std::iter::repeat(f32::NAN)).take(count)
}

/// What the line renderer draws: each line with data as its line index
/// followed by the bits of its `profile_length` samples.
pub struct LinePayload {
    pub words: Vec<u32>,
    pub shape: LineShape,
}

impl OctantApp {
    /// The base layer's line data and how its lines lie in it.
    pub fn line_layout(&self) -> (&[f32], LineLayout) {
        let data = &self.layers.base.data;
        let pick = (!self.line_plot_all_series).then_some(self.line_profile_slice_idx);
        if self.line_profile_dim_idx == 2
            && let Some(v) = data.volume.as_ref().filter(|v| v.depth > 1)
        {
            let Some(pixels) = v.width.checked_mul(v.height) else {
                return (&[], LineLayout::default());
            };
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
        let words = layout.line_count.checked_mul(len.saturating_add(1));
        let mut payload = Vec::with_capacity(words.unwrap_or(0));
        let mut drawn = 0;
        for line in 0..layout.line_count {
            if !layout.row(values, line).any(f32::is_finite) {
                continue;
            }
            payload.push(line as u32);
            payload.extend(layout.row(values, line).map(f32::to_bits));
            drawn += 1;
        }
        LinePayload {
            words: payload,
            shape: shape_of(layout, drawn),
        }
    }

    /// Bytes of the payload `line_payload` would build, without building it.
    fn line_payload_bytes(&self) -> Option<u64> {
        let (values, layout) = self.line_layout();
        let drawn = (0..layout.line_count)
            .filter(|&line| layout.row(values, line).any(f32::is_finite))
            .count();
        let words = drawn.checked_mul(layout.profile_length.checked_add(1)?)?;
        u64::try_from(words).ok()?.checked_mul(4)
    }

    /// Identifies the payload `line_payload` builds: the base layer's data
    /// version and the layout of its lines.
    pub fn line_payload_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        (self.layers.base.data.version, self.line_layout().1).hash(&mut hasher);
        hasher.finish()
    }

    /// The shape of the payload `renderer` draws, building and uploading it
    /// first only when the data or its layout changed since the last upload.
    /// A payload past the device's buffer limit is never built.
    pub fn upload_line_payload(&self, renderer: &LineRenderer) -> LineShape {
        let key = self.line_payload_key();
        if let Some(shape) = renderer.payload_shape(key) {
            return shape;
        }
        let Some(state) = &self.wgpu_render_state else {
            return LineShape::default();
        };
        let (device, queue) = (&state.device, &state.queue);
        let limit = LineRenderer::payload_limit(device);
        let layout = self.line_layout().1;
        // Count the lines with data only when drawing all of them would not fit.
        let bytes = match layout.max_payload_bytes() {
            Some(most) if most <= limit => most,
            _ => self.line_payload_bytes().unwrap_or(u64::MAX),
        };
        if bytes > limit {
            renderer.skip_payload(key, shape_of(layout, 0), bytes, limit);
            return renderer.payload_shape(key).unwrap_or_default();
        }
        let payload = self.line_payload();
        renderer.upload_payload(device, queue, key, &payload.words, payload.shape);
        payload.shape
    }
}

/// The shape of a payload of `layout` with `drawn` lines.
fn shape_of(layout: LineLayout, drawn: u32) -> LineShape {
    LineShape {
        profile_length: layout.profile_length as u32,
        drawn_lines: drawn,
        line_count: layout.line_count as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::LineLayout;
    use crate::app::OctantApp;

    #[test]
    fn the_payload_key_follows_the_data_and_its_layout() {
        let mut app = OctantApp::default();
        app.layers.base.data.matrix = Some(crate::data::MatrixData::new(
            3,
            2,
            vec![1.0; 6],
            1.0,
            1.0,
            "rows".to_string(),
            1,
        ));
        let key = app.line_payload_key();
        assert_eq!(app.line_payload_key(), key, "nothing changed");
        app.layers.base.data.touch();
        let touched = app.line_payload_key();
        assert_ne!(touched, key, "the data changed");
        app.line_profile_slice_idx = 1;
        let last = app.line_payload_key();
        assert_ne!(last, touched, "another line picked");
        app.line_profile_slice_idx = 5;
        assert_eq!(
            app.line_payload_key(),
            last,
            "past the last line: the same line"
        );
        app.line_plot_all_series = true;
        let all = app.line_payload_key();
        app.line_profile_slice_idx = 0;
        assert_eq!(app.line_payload_key(), all, "every line drawn: no pick");
        app.line_profile_dim_idx = 1;
        assert_ne!(app.line_payload_key(), all, "lines along another axis");
    }

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
    fn rows_and_columns_read_in_order_with_nan_past_the_end() {
        let rows = LineLayout::lines(3, 2, (3, 1), None);
        assert_eq!(rows.row(&VALUES, 1).collect::<Vec<_>>(), [4.0, 5.0, 6.0]);
        assert_eq!(rows.column(&VALUES, 2).collect::<Vec<_>>(), [3.0, 6.0]);
        let short: Vec<f32> = rows.row(&VALUES[..5], 1).collect();
        assert_eq!(short[..2], [4.0, 5.0]);
        assert!(short[2].is_nan());
        assert_eq!(rows.max_payload_bytes(), Some(2 * 4 * 4));
    }

    #[test]
    fn a_picked_line_is_clamped_and_short_data_reads_nan() {
        let row = LineLayout::lines(3, 2, (3, 1), Some(9));
        assert_eq!((row.line_count, row.value(&VALUES, 0, 0)), (1, 4.0));
        assert!(row.value(&VALUES[..4], 0, 2).is_nan());
        assert_eq!(LineLayout::lines(3, 0, (3, 1), Some(0)).line_count, 0);
    }
}
