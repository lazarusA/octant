//! Raw channel values behind a projected 2D composite, read back for hover readouts.

use std::sync::Arc;

use super::cmyk::is_cmyk_block;
use crate::data::octant_block::OctantBlock;

/// Local index of global channel `channel` in a block whose channel window starts at
/// `start` and holds `count` channels; out-of-window channels clamp to the last one.
pub(crate) fn local_channel(channel: usize, start: usize, count: usize) -> usize {
    if channel >= start && channel < start + count {
        channel - start
    } else {
        channel.min(count.saturating_sub(1))
    }
}

/// The block window a 2D composite was sliced from. It shares the block's value buffer, so
/// keeping it costs a few small index vectors.
#[derive(Clone, Debug)]
pub struct CompositeProbe {
    values: Arc<[f32]>,
    shape: Vec<usize>,
    strides: Vec<usize>,
    fixed: Vec<usize>,
    channel_start: usize,
    /// The block holds CMYK inks, which the composite converts instead of mapping to RGB.
    cmyk: bool,
    c_dim: usize,
    x_dim: usize,
    y_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
}

impl CompositeProbe {
    /// Records the window `slice_*_composite_nd` reads; ranges and fixed indices are local.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block: &OctantBlock,
        c_dim: usize,
        x_dim: usize,
        y_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        fixed_indices: &[usize],
    ) -> Self {
        Self {
            values: Arc::clone(&block.values),
            shape: block.shape.clone(),
            strides: block.strides.clone(),
            fixed: fixed_indices.to_vec(),
            channel_start: block.origin.get(c_dim).copied().unwrap_or(0),
            cmyk: is_cmyk_block(block, c_dim),
            c_dim,
            x_dim,
            y_dim,
            x_range,
            y_range,
        }
    }

    /// The global C, M, Y and K channels when the composite converts CMYK inks.
    pub fn cmyk_channels(&self) -> Option<[usize; 4]> {
        let start = self.channel_start;
        self.cmyk
            .then_some([start, start + 1, start + 2, start + 3])
    }

    /// The global channel an RGB composite actually draws when asked for `channel`.
    pub fn resolve_channel(&self, channel: usize) -> usize {
        let count = self.shape.get(self.c_dim).copied().unwrap_or(1);
        self.channel_start + local_channel(channel, self.channel_start, count)
    }

    /// Raw value of global channel `channel` at composite pixel `(px, py)`, or `None` when
    /// either lies outside the window.
    pub fn sample(&self, channel: usize, px: usize, py: usize) -> Option<f32> {
        let width = self.x_range.1.saturating_sub(self.x_range.0);
        let height = self.y_range.1.saturating_sub(self.y_range.0);
        if px >= width || py >= height {
            return None;
        }
        let count = self.shape.get(self.c_dim).copied().unwrap_or(0);
        let local = channel
            .checked_sub(self.channel_start)
            .filter(|&c| c < count)?;
        let mut offset = 0usize;
        for (dim, (&size, &stride)) in self.shape.iter().zip(&self.strides).enumerate() {
            let idx = if dim == self.c_dim {
                local
            } else if dim == self.x_dim {
                self.x_range.0 + px
            } else if dim == self.y_dim {
                self.y_range.0 + py
            } else {
                self.fixed.get(dim).copied().unwrap_or(0)
            };
            if idx >= size {
                return None;
            }
            offset = offset.checked_add(idx.checked_mul(stride)?)?;
        }
        self.values.get(offset).copied()
    }
}
