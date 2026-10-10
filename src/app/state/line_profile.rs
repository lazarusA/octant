//! Where a line plot's lines live in the base layer's data: one `LineLayout`
//! read by the GPU payload, the hover and the series colorbar, so they agree
//! on which line is which without building the payload. The payload is built
//! and uploaded only when its key (`line_payload_key`) changes.

use super::app_state::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::plots::{LineRenderer, LineShape};
use crate::ui::temp_cache::hash_key;

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

    /// The lines of `values` with data, in order: the lines a payload holds.
    pub(super) fn lines_with_data(&self, values: &[f32]) -> Vec<u32> {
        (0..self.line_count)
            .filter(|&l| self.has_data(values, l))
            .filter_map(|l| u32::try_from(l).ok())
            .collect()
    }

    /// The payload of lines `drawn` (`lines_with_data`) of `values`.
    fn payload(&self, values: &[f32], drawn: &[u32]) -> LinePayload {
        let words = self.payload_bytes(drawn.len()).unwrap_or(0) / 4;
        let mut payload = Vec::with_capacity(usize::try_from(words).unwrap_or(0));
        for &line in drawn {
            payload.push(line);
            payload.extend(self.row(values, line as usize).map(f32::to_bits));
        }
        LinePayload {
            words: payload,
            shape: self.shape(u32::try_from(drawn.len()).unwrap_or(u32::MAX)),
        }
    }

    /// The shape of a payload of this layout with `drawn` lines.
    fn shape(&self, drawn: u32) -> LineShape {
        let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        LineShape {
            profile_length: to_u32(self.profile_length),
            drawn_lines: drawn,
            line_count: to_u32(self.line_count),
        }
    }
}

/// The data lines are read from, at its version (`LayerData::matrix_version`
/// or `volume_version`): the two count apart, so the key names which one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LineData {
    Matrix(u64),
    Volume(u64),
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

    /// The line data, its layout, and which data it is at what version.
    fn line_source(&self) -> (&[f32], LineLayout, LineData) {
        let data = &self.layers.base.data;
        let pick = (!self.plot_configs.line.all_series)
            .then_some(self.plot_configs.line.profile_slice_idx);
        if self.plot_configs.line.profile_dim_idx == 2
            && let Some(v) = data.volume.as_ref().filter(|v| v.depth > 1)
        {
            let Some(pixels) = v.width.checked_mul(v.height) else {
                return (
                    &[],
                    LineLayout::default(),
                    LineData::Volume(data.volume_version),
                );
            };
            let layout = LineLayout::lines(v.depth, pixels, (1, pixels), pick);
            (&v.values, layout, LineData::Volume(data.volume_version))
        } else if let Some(m) = &data.matrix {
            let layout = match self.plot_configs.line.profile_dim_idx {
                0 => LineLayout::lines(m.width, m.height, (m.width, 1), pick),
                _ => LineLayout::lines(m.height, m.width, (1, m.width), pick),
            };
            (&m.values, layout, LineData::Matrix(data.matrix_version))
        } else {
            (
                &[],
                LineLayout::default(),
                LineData::Matrix(data.matrix_version),
            )
        }
    }

    /// The renderer's payload: lines without a finite value are left out.
    pub fn line_payload(&self) -> LinePayload {
        let (values, layout) = self.line_layout();
        layout.payload(values, &layout.lines_with_data(values))
    }

    /// Identifies the payload `line_payload` builds: the data it reads (the
    /// matrix or the volume, at its version) and the layout of its lines.
    pub fn line_payload_key(&self) -> u64 {
        let (_, layout, source) = self.line_source();
        payload_key(layout, source)
    }

    /// The shape of the payload `layer`'s line renderer draws, building and
    /// uploading it first only when the data or its layout changed since the
    /// last upload. Only the base layer draws lines. A payload past the
    /// device's buffer limit is never built: it is refused, and reported once
    /// until the layout changes.
    pub fn upload_line_payload(&self, layer: &Layer) -> LineShape {
        let Some(renderer) = layer.renderers.line.as_deref() else {
            return LineShape::default();
        };
        let Some(state) = self.wgpu_render_state.as_ref() else {
            return LineShape::default();
        };
        debug_assert_eq!(layer.id(), LayerId::BASE, "only the base layer draws lines");
        let (values, layout, source) = self.line_source();
        let key = payload_key(layout, source);
        if let Some(shape) = renderer.payload_shape(key) {
            return shape;
        }
        if renderer.refuses(key) {
            return layout.shape(0);
        }
        let drawn = layout.lines_with_data(values);
        let bytes = layout.payload_bytes(drawn.len()).unwrap_or(u64::MAX);
        let limit = LineRenderer::payload_limit(&state.device);
        if bytes > limit {
            renderer.refuse(key, hash_key(layout), bytes, limit);
            return layout.shape(0);
        }
        let payload = layout.payload(values, &drawn);
        let (device, queue) = (&state.device, &state.queue);
        renderer.upload_payload(device, queue, key, &payload.words, payload.shape);
        payload.shape
    }
}

/// The key of the payload of `layout` over `source`.
fn payload_key(layout: LineLayout, source: LineData) -> u64 {
    hash_key((source, layout))
}
