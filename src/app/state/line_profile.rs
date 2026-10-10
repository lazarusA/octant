//! Where a line plot's lines live in the base layer's data: one `LineLayout`
//! read by the GPU payload, the hover and the series colorbar, so they agree
//! on which line is which without building the payload. The payload is built
//! and uploaded only when its key (`line_payload_key`) changes.

use std::hash::{DefaultHasher, Hash, Hasher};

use super::app_state::OctantApp;
use crate::plots::{LineRenderer, LineShape};

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
            shape: LineShape {
                profile_length: len as u32,
                drawn_lines: drawn,
                line_count: layout.line_count as u32,
            },
        }
    }

    /// Identifies the payload `line_payload` builds: the base layer's data
    /// version and the choices that lay its lines out.
    pub fn line_payload_key(&self) -> u64 {
        let pick = (!self.line_plot_all_series).then_some(self.line_profile_slice_idx);
        let mut hasher = DefaultHasher::new();
        (
            self.layers.base.data.version,
            self.line_profile_dim_idx,
            pick,
        )
            .hash(&mut hasher);
        hasher.finish()
    }

    /// The shape of the payload `renderer` draws, building and uploading it
    /// first only when the data or its layout changed since the last upload.
    pub fn upload_line_payload(&self, renderer: &LineRenderer) -> LineShape {
        let key = self.line_payload_key();
        if let Some(shape) = renderer.payload_shape(key) {
            return shape;
        }
        let payload = self.line_payload();
        if let Some(state) = &self.wgpu_render_state {
            let (device, queue) = (&state.device, &state.queue);
            renderer.upload_payload(device, queue, key, &payload.values, payload.shape);
        }
        payload.shape
    }
}

#[cfg(test)]
mod tests {
    use super::LineLayout;
    use crate::app::OctantApp;

    #[test]
    fn the_payload_key_follows_the_data_and_its_layout() {
        let mut app = OctantApp::default();
        let key = app.line_payload_key();
        assert_eq!(app.line_payload_key(), key, "nothing changed");
        app.layers.base.data.touch();
        let touched = app.line_payload_key();
        assert_ne!(touched, key, "the data changed");
        app.line_profile_slice_idx += 1;
        assert_ne!(app.line_payload_key(), touched, "another line picked");
        app.line_plot_all_series = true;
        let all = app.line_payload_key();
        app.line_profile_slice_idx += 1;
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
    fn a_picked_line_is_clamped_and_short_data_reads_nan() {
        let row = LineLayout::lines(3, 2, (3, 1), Some(9));
        assert_eq!((row.line_count, row.value(&VALUES, 0, 0)), (1, 4.0));
        assert!(row.value(&VALUES[..4], 0, 2).is_nan());
        assert_eq!(LineLayout::lines(3, 0, (3, 1), Some(0)).line_count, 0);
    }
}
