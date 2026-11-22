//! Dropping a plot's bricks and building the next plot's (`setup.rs`
//! `build_lod`), with their renderer: a new one, or the last one when its
//! atlas and table fit (the next step, or a composite recolored).

use std::sync::Arc;

use super::setup::Off;
use super::{Lod, VolumeLodState};
use crate::app::OctantApp;
use crate::i18n::{NoticesKey, filled, tr};
use crate::plots::VolumeRenderer;
use crate::ui::toast::Severity;

/// Storage buffers the bricked shader reads in its fragment stage (slot
/// ranges, sub-block ranges, ray feedback).
const FRAGMENT_STORAGE_BUFFERS: u32 = 3;

impl OctantApp {
    /// Drops the bricks of the last plot (cancelling their reads) and
    /// builds those of `context`'s, asking for its roots at once; into
    /// `reuse`'s atlas when it fits.
    pub(super) fn reset_volume_lod(
        &mut self,
        context: Option<u64>,
        reuse: Option<Arc<VolumeRenderer>>,
    ) {
        self.cancel_brick_reads();
        let colored = context.map(|_| super::composite::colors_of(self));
        let warned = self.layers.base.data.volume_lod.warned.take();
        self.layers.base.data.volume_lod = VolumeLodState {
            context,
            colored,
            warned,
            ..Default::default()
        };
        self.layers.base.renderers.volume_bricked = None;
        if context.is_none() {
            return;
        }
        let mut lod = match self.build_lod() {
            Some(Ok(lod)) => lod,
            Some(Err(off)) => return self.warn_lod_off(off),
            None => return,
        };
        lod.queue = lod.roots.iter().copied().collect();
        if self.wgpu_render_state.is_some() {
            match self.bricked_renderer(&lod, reuse) {
                Ok(renderer) => self.layers.base.renderers.volume_bricked = Some(renderer),
                Err(off) => return self.warn_lod_off(off),
            }
        }
        self.layers.base.data.volume_lod.lod = Some(lod);
    }

    /// Cancels the reads of the bricks being dropped: nobody takes them
    /// (their failures would show as a plot's), and they hold workers the
    /// next plot needs.
    fn cancel_brick_reads(&mut self) {
        let lod = self.layers.base.data.volume_lod.lod.as_mut();
        for (key, _) in lod
            .map(|l| std::mem::take(&mut l.pending))
            .unwrap_or_default()
        {
            // A root over a whole small level reads the plot's own block.
            if self.layers.find_by_key(&key).is_none() {
                self.block_prefetcher.cancel(&key);
            }
        }
    }

    /// Tells why the plot stays dense, once per variable and reason (a new
    /// step builds anew).
    fn warn_lod_off(&mut self, off: Off) {
        let var = self.plotted().variable_info().map(|v| v.name.clone());
        let warned = Some((var.unwrap_or_default(), off));
        let state = &mut self.layers.base.data.volume_lod;
        if state.warned == warned {
            return;
        }
        state.warned = warned;
        let detail = match off {
            Off::Roots(roots, slots) => {
                filled(tr(NoticesKey::VolumeLodOffDetail), &[&roots, &slots])
            }
            Off::Uneven => tr(NoticesKey::VolumeLodUnevenDetail).to_owned(),
            Off::WideIntegers => tr(NoticesKey::VolumeLodWideIntegersDetail).to_owned(),
            Off::Unsupported => tr(NoticesKey::VolumeLodUnsupportedDetail).to_owned(),
            Off::TooLarge([x, y, z], max) => filled(
                tr(NoticesKey::VolumeTextureTooLargeDetail),
                &[&x, &y, &z, &max],
            ),
        };
        self.notify(Severity::Warning, tr(NoticesKey::VolumeLodOff), detail);
    }

    /// The renderer of `lod`'s bricks: `reuse`, its table cleared, when its
    /// atlas and table fit them, else a new one; `Err` when none can draw.
    fn bricked_renderer(
        &mut self,
        lod: &Lod,
        reuse: Option<Arc<VolumeRenderer>>,
    ) -> Result<Arc<VolumeRenderer>, Off> {
        let rs = self.wgpu_render_state.clone().ok_or(Off::Unsupported)?;
        // Rays write what they see from the fragment stage (WebGL and some
        // GL ES devices cannot).
        let flags = rs.adapter.get_downlevel_capabilities().flags;
        if !flags.contains(wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE)
            || rs.device.limits().max_storage_buffers_per_shader_stage < FRAGMENT_STORAGE_BUFFERS
        {
            return Err(Off::Unsupported);
        }
        let grid = lod.bricks().grid();
        let fits = |r: &Arc<VolumeRenderer>| {
            r.bricks()
                .is_some_and(|b| b.fits(lod.slots, grid, lod.texel))
        };
        let renderer = match reuse.filter(fits) {
            Some(r) => {
                if let Some(bricks) = r.bricks() {
                    bricks.clear_table(&rs.queue);
                }
                // The last plot's frame must not show while this one builds.
                r.forget_frames();
                r
            }
            None => {
                let target = (lod.slots, grid, lod.texel);
                let new =
                    VolumeRenderer::new_bricked(&rs.device, &rs.queue, rs.target_format, target);
                let max = rs.device.limits().max_texture_dimension_3d;
                Arc::new(new.ok_or(Off::TooLarge(grid, max))?)
            }
        };
        if let Some(bricks) = renderer.bricks() {
            bricks.set_uniforms(&rs.queue, &lod.uniforms());
        }
        Ok(renderer)
    }
}
