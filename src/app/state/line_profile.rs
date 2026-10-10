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
    /// The line drawn alone and how many there are, when one is picked.
    pub pick: Option<(usize, usize)>,
    first: usize,
    line_stride: usize,
    sample_stride: usize,
}

impl LineLayout {
    /// Every line of `count` (All Lines Series), or only line `pick` of them.
    pub(super) fn lines(
        len: usize,
        count: usize,
        strides: (usize, usize),
        pick: Option<usize>,
    ) -> Self {
        let (line_stride, sample_stride) = strides;
        let pick = pick.filter(|_| count > 0).map(|line| line.min(count - 1));
        // An index past any data (on overflow) reads NaN.
        let first = pick.map_or(Some(0), |line| line.checked_mul(line_stride));
        Self {
            profile_length: len,
            line_count: if pick.is_some() { 1 } else { count },
            pick: pick.map(|line| (line, count)),
            first: first.unwrap_or(usize::MAX),
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

    /// Whether line `line` has a finite sample: only those are drawn.
    pub(super) fn has_data(&self, values: &[f32], line: usize) -> bool {
        self.row(values, line).any(f32::is_finite)
    }

    /// Bytes of a payload of `lines` lines: each its index and its samples.
    pub(super) fn payload_bytes(&self, lines: usize) -> Option<u64> {
        let words = lines.checked_mul(self.profile_length.checked_add(1)?)?;
        u64::try_from(words).ok()?.checked_mul(4)
    }

    /// The shape of a payload of this layout with `drawn` lines.
    fn shape(&self, drawn: u32) -> LineShape {
        LineShape {
            profile_length: self.profile_length as u32,
            drawn_lines: drawn,
            line_count: self.line_count as u32,
        }
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
        let (values, layout, _) = self.line_source();
        (values, layout)
    }

    /// The line data, its layout, and the version of the data it comes from
    /// (`LayerData::matrix_version` or `volume_version`).
    fn line_source(&self) -> (&[f32], LineLayout, u64) {
        let data = &self.layers.base.data;
        let pick = (!self.line_plot_all_series).then_some(self.line_profile_slice_idx);
        if self.line_profile_dim_idx == 2
            && let Some(v) = data.volume.as_ref().filter(|v| v.depth > 1)
        {
            let Some(pixels) = v.width.checked_mul(v.height) else {
                return (&[], LineLayout::default(), data.volume_version);
            };
            let layout = LineLayout::lines(v.depth, pixels, (1, pixels), pick);
            (&v.values, layout, data.volume_version)
        } else if let Some(m) = &data.matrix {
            let layout = match self.line_profile_dim_idx {
                0 => LineLayout::lines(m.width, m.height, (m.width, 1), pick),
                _ => LineLayout::lines(m.height, m.width, (1, m.width), pick),
            };
            (&m.values, layout, data.matrix_version)
        } else {
            (&[], LineLayout::default(), data.matrix_version)
        }
    }

    /// The renderer's payload: lines without a finite value are left out.
    pub fn line_payload(&self) -> LinePayload {
        let (values, layout) = self.line_layout();
        let words = layout
            .line_count
            .checked_mul(layout.profile_length.saturating_add(1));
        let mut payload = Vec::with_capacity(words.unwrap_or(0));
        let mut drawn = 0;
        for line in (0..layout.line_count).filter(|&l| layout.has_data(values, l)) {
            payload.push(line as u32);
            payload.extend(layout.row(values, line).map(f32::to_bits));
            drawn += 1;
        }
        LinePayload {
            words: payload,
            shape: layout.shape(drawn),
        }
    }

    /// Identifies the payload `line_payload` builds: the version of the data
    /// it reads and the layout of its lines.
    pub fn line_payload_key(&self) -> u64 {
        let (_, layout, version) = self.line_source();
        hash_of((version, layout))
    }

    /// The shape of the payload `renderer` draws, building and uploading it
    /// first only when the data or its layout changed since the last upload.
    /// A payload past the device's buffer limit is never built: its layout is
    /// refused (and reported) once, until the layout changes.
    pub fn upload_line_payload(&self, renderer: &LineRenderer) -> LineShape {
        let (values, layout, version) = self.line_source();
        let key = hash_of((version, layout));
        if let Some(shape) = renderer.payload_shape(key) {
            return shape;
        }
        let layout_key = hash_of(layout);
        let Some(state) = &self.wgpu_render_state else {
            return LineShape::default();
        };
        if renderer.refuses(layout_key) {
            return layout.shape(0);
        }
        let limit = LineRenderer::payload_limit(&state.device);
        // Count the lines with data only when drawing all of them would not fit.
        let bytes = match layout.payload_bytes(layout.line_count) {
            Some(most) if most <= limit => most,
            _ => {
                let drawn = (0..layout.line_count).filter(|&l| layout.has_data(values, l));
                layout.payload_bytes(drawn.count()).unwrap_or(u64::MAX)
            }
        };
        if bytes > limit {
            renderer.refuse(layout_key, bytes, limit);
            return layout.shape(0);
        }
        let payload = self.line_payload();
        let (device, queue) = (&state.device, &state.queue);
        renderer.upload_payload(device, queue, key, &payload.words, payload.shape);
        payload.shape
    }
}

fn hash_of(value: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
