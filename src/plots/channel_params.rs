//! The uniforms of `common/channel_blend.wgsl`: a multi-channel composite's
//! drawn channels, built from its style every frame.

use bytemuck::{Pod, Zeroable};

use crate::data::slicing::{ChannelColorConfig, CompositeStretch};

/// The most channels one plane buffer holds (`ChannelParams::channels`).
pub const MAX_CHANNELS: usize = 16;

/// One drawn channel (`Channel` of `common/channel_blend.wgsl`).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct ChannelGpu {
    plane: u32,
    family: u32,
    inverted: u32,
    is_i8: u32,
    tint: [f32; 4],
    scale: f32,
    offset: f32,
    _pad: [f32; 2],
    curve: [f32; 4],
}

/// The drawn channels and the NaN color (`ChannelParams` of `common/channel_blend.wgsl`).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct ChannelParams {
    count: u32,
    plane_len: u32,
    use_nan: u32,
    _pad: u32,
    nan_color: [f32; 4],
    channels: [ChannelGpu; MAX_CHANNELS],
}

impl ChannelParams {
    /// The channels of `configs` drawn from planes of channels `c_start..`
    /// (`planes` of them), as `drawn_channels` picks them: the ticked ones,
    /// else the first; those without a window stretched as `stretch`
    /// recorded. Builds nothing on the heap, so it runs every frame.
    pub fn of(
        configs: &[ChannelColorConfig],
        stretch: &CompositeStretch,
        (c_start, planes): (usize, usize),
        plane_len: u32,
        nan_color: Option<[f32; 4]>,
    ) -> Self {
        let mut out = Self {
            count: 0,
            plane_len,
            use_nan: u32::from(nan_color.is_some()),
            _pad: 0,
            nan_color: nan_color.unwrap_or_default(),
            channels: [ChannelGpu::default(); MAX_CHANNELS],
        };
        let in_window = |c: &&ChannelColorConfig| (c_start..c_start + planes).contains(&c.index);
        for cfg in configs.iter().filter(in_window).filter(|c| c.visible) {
            out.push(cfg, c_start, stretch);
        }
        if out.count == 0
            && let Some(first) = configs.iter().find(|c| c.index == c_start && planes > 0)
        {
            out.push(first, c_start, stretch);
        }
        out
    }

    /// The planes drawn, in order.
    #[cfg(test)]
    pub(crate) fn planes(&self) -> Vec<u32> {
        (self.channels.iter().take(self.count as usize))
            .map(|c| c.plane)
            .collect()
    }

    fn push(&mut self, cfg: &ChannelColorConfig, c_start: usize, stretch: &CompositeStretch) {
        let (Some(slot), Some(s)) = (
            self.channels.get_mut(self.count as usize),
            stretch.of_channel(cfg.index, cfg.window),
        ) else {
            return;
        };
        let (family, inverted, [k, top, span]) = cfg.display.curve().gpu_terms();
        let [r, g, b] = cfg.color_rgb.map(f32::from);
        *slot = ChannelGpu {
            plane: (cfg.index - c_start) as u32,
            family,
            inverted: u32::from(inverted),
            is_i8: u32::from(s.is_i8),
            tint: [r, g, b, 0.0],
            scale: s.scale,
            offset: s.offset,
            _pad: [0.0; 2],
            curve: [k, top, span, 0.0],
        };
        self.count += 1;
    }
}
