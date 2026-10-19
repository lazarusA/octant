//! How a 2D composite maps each channel's raw values onto color intensities.
//! Without a display window a channel is stretched over the values it was
//! sliced from, so a finer tile of the same plot (`app::detail`) reuses its
//! plot's stretches instead of computing its own, keeping colors continuous
//! across the tile's edges.

use super::utils::compute_channel_normalization;

/// One channel's normalization: `(raw - offset) * scale`, after reading the
/// raw value's int8 bit pattern as unsigned when `is_i8`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stretch {
    pub scale: f32,
    pub offset: f32,
    pub is_i8: bool,
}

impl Stretch {
    /// The stretch of `values` (or of `window`, in raw values) onto `0..target`.
    pub fn of(values: &[f32], window: Option<(f32, f32)>, target: f32) -> Self {
        let (scale, offset, is_i8) = compute_channel_normalization(values, window, target);
        Self {
            scale,
            offset,
            is_i8,
        }
    }
}

/// The stretches a composite was drawn with: one per global channel, or one
/// shared by CMYK's inks.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompositeStretch {
    channels: Vec<(usize, Stretch)>,
    inks: Option<Stretch>,
}

impl CompositeStretch {
    /// Channel `channel`'s stretch: `window`'s when it has one, else the one
    /// recorded, else `values`' onto `0..target`, recorded.
    pub fn channel(
        &mut self,
        channel: usize,
        values: &[f32],
        window: Option<(f32, f32)>,
        target: f32,
    ) -> Stretch {
        if window.is_some() {
            return Stretch::of(values, window, target);
        }
        if let Some(&(_, s)) = self.channels.iter().find(|(c, _)| *c == channel) {
            return s;
        }
        let s = Stretch::of(values, None, target);
        self.channels.push((channel, s));
        s
    }

    /// The inks' shared stretch: the one recorded, else `make()`, recorded.
    pub fn inks(&mut self, make: impl FnOnce() -> Stretch) -> Stretch {
        *self.inks.get_or_insert_with(make)
    }
}
