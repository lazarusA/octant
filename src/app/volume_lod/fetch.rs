//! Brick reads: each brick is an ordinary
//! block request of its level over the brick's cells and apron, through the
//! block cache and the prefetcher (so the status bar's spinner shows them),
//! at most `IN_FLIGHT` at once so decoding leaves the UI its share of the
//! CPU. Arrivals wait in a queue and are placed in batches every
//! `PLACE_EVERY` seconds within `PLACE_BUDGET` of the frame (each batch
//! re-renders the volume once, `place.rs`).

use crate::app::OctantApp;
use crate::app::detail::LevelRead;
use crate::app::layers::LayerId;
use crate::app::multiscale::to_level;
use crate::data::multiscale::bricks::BrickKey;
use crate::data::{BlockCacheKey, BlockRequest, OctantBlock};

/// Brick reads running at once.
const IN_FLIGHT: usize = 8;
/// Seconds between batches of placed bricks.
pub(super) const PLACE_EVERY: f64 = 0.1;
/// Time a frame spends placing bricks.
const PLACE_BUDGET: std::time::Duration = std::time::Duration::from_millis(6);

impl OctantApp {
    /// Reads queued bricks while fewer than `IN_FLIGHT` are running and a
    /// worker is free; cached ones join the arrivals at once.
    pub(super) fn request_queued_bricks(&mut self) {
        loop {
            let lod = self.layers.base.data.volume_lod.lod.as_mut();
            let Some(lod) = lod.filter(|l| l.pending.len() < IN_FLIGHT) else {
                return;
            };
            let Some(key) = lod.queue.pop_front() else {
                return;
            };
            if !self.request_brick(key) {
                if let Some(lod) = self.layers.base.data.volume_lod.lod.as_mut() {
                    lod.queue.push_front(key);
                }
                return;
            }
        }
    }

    /// Reads `key`; whether it was taken (else every worker is busy).
    fn request_brick(&mut self, key: BrickKey) -> bool {
        let Some(read) = self.brick_read(key) else {
            return true;
        };
        let cached = self.block_cache.get(&read.key);
        let request = BlockRequest::new(read.handle.clone(), read.slice.clone());
        let taken = cached.is_some()
            || self.block_prefetcher.request(request, &self.block_cache)
            || self.block_prefetcher.is_pending(&read.key);
        let Some(lod) = self.layers.base.data.volume_lod.lod.as_mut() else {
            return taken;
        };
        match cached {
            Some(block) => lod.arrived.push_back((key, read, block)),
            None if taken => {
                lod.pending.insert(read.key.clone(), (key, read));
            }
            None => {}
        }
        taken
    }

    /// The block request reading `key`'s cells and apron.
    fn brick_read(&self, key: BrickKey) -> Option<LevelRead> {
        let lod = self.layers.base.data.volume_lod.lod.as_ref()?;
        let window = lod.bricks().window(key)?;
        let mut sel = to_level(self.plotted(), usize::from(key.level))?;
        for (i, w) in window.iter().enumerate() {
            let (start, end) = (w.start as usize, w.end.saturating_sub(1) as usize);
            sel.set_range(lod.dims[i], (start, end), start);
        }
        self.level_read(LayerId::BASE, sel)
    }

    /// Takes a fetch result when it is a pending brick's; whether it was.
    pub(crate) fn take_brick_result(
        &mut self,
        key: &BlockCacheKey,
        result: &Result<OctantBlock, String>,
    ) -> bool {
        let lod = self.layers.base.data.volume_lod.lod.as_mut();
        let Some((brick, read)) = lod.and_then(|l| l.pending.remove(key)) else {
            return false;
        };
        match result {
            Ok(block) => {
                // Composites recolor from the plotted block: keep it newer
                // than the bricks, which would otherwise evict it first.
                self.keep_plotted_block();
                self.block_cache.put(key.clone(), block.clone());
                if let Some(lod) = self.layers.base.data.volume_lod.lod.as_mut() {
                    lod.arrived.push_back((brick, read, block.clone()));
                }
            }
            // Asked again when the next resting view still wants it.
            Err(e) => self.report_brick_failure(brick, e),
        }
        true
    }

    /// Logs a failed read, and reports the first of a plot.
    fn report_brick_failure(&mut self, brick: BrickKey, e: &str) {
        let lod = self.layers.base.data.volume_lod.lod.as_mut();
        let Some(lod) = lod.filter(|l| !l.failure_reported) else {
            log::warn!("Brick {brick:?} failed: {e}");
            return;
        };
        lod.failure_reported = true;
        let title = crate::i18n::tr(crate::i18n::NoticesKey::VolumeLodBrickFailed);
        self.notify(crate::ui::toast::Severity::Warning, title, e);
    }

    /// Asks again for bricks whose read was lost (an abort drops reads
    /// without a result): neither running nor cached.
    pub(super) fn requeue_lost_bricks(&mut self) {
        let (prefetcher, cache) = (&self.block_prefetcher, &self.block_cache);
        let Some(lod) = self.layers.base.data.volume_lod.lod.as_mut() else {
            return;
        };
        let lost: Vec<BlockCacheKey> = lod
            .pending
            .keys()
            .filter(|k| !prefetcher.is_pending(k) && !cache.contains(k))
            .cloned()
            .collect();
        for key in lost {
            if let Some((brick, _)) = lod.pending.remove(&key) {
                lod.queue.push_front(brick);
            }
        }
    }

    /// Cancels the reads of bricks the latest selection no longer wants
    /// (the view moved on): queued ones never start, running ones end into
    /// the block cache, and their places go to the bricks wanted now. A read
    /// a layer waits on (a brick window equal to a plotted one, joined
    /// through `is_pending` in `request_brick`) is left running.
    pub(super) fn drop_unwanted_reads(&mut self) {
        let Some(lod) = self.layers.base.data.volume_lod.lod.as_mut() else {
            return;
        };
        let unwanted: Vec<BlockCacheKey> = lod
            .pending
            .iter()
            .filter(|(_, (brick, _))| !lod.wanted.contains(brick) && !lod.roots.contains(brick))
            .map(|(key, _)| key.clone())
            .collect();
        for key in &unwanted {
            lod.pending.remove(key);
        }
        for key in unwanted {
            if self.layers.find_by_key(&key).is_none() {
                self.block_prefetcher.cancel(&key);
            }
        }
    }

    /// Places arrived bricks, a batch every `PLACE_EVERY` seconds (egui time
    /// `now`) within `PLACE_BUDGET`.
    pub(super) fn place_arrivals(&mut self, now: f64) {
        let cache = &self.block_cache;
        let state = &mut self.layers.base.data.volume_lod;
        let generation = state.generation;
        let Some(lod) = state.lod.as_mut() else {
            return;
        };
        if lod.arrived.is_empty() || now - lod.placed_at < PLACE_EVERY {
            return;
        }
        lod.placed_at = now;
        let rs = self.wgpu_render_state.clone();
        let gpu = self.layers.base.renderers.volume_bricked.as_ref();
        let gpu = gpu
            .and_then(|r| r.bricks())
            .zip(rs.as_ref().map(|rs| &rs.queue));
        let start = web_time::Instant::now();
        let mut unreadable = None;
        while start.elapsed() < PLACE_BUDGET {
            let Some((key, read, block)) = lod.arrived.pop_front() else {
                break;
            };
            // Read for an earlier view and no longer wanted: evictable.
            let wanted_by = if lod.wanted.contains(&key) {
                generation
            } else {
                0
            };
            let name = block.variable_name.clone();
            if !lod.place(key, &read, block, wanted_by, (gpu, cache)) {
                // Never asked again (it would read the same).
                lod.unreadable.insert(key);
                unreadable = Some((key, name));
            }
        }
        lod.refresh_finest();
        if let Some((key, name)) = unreadable {
            self.report_brick_failure(key, &name);
        }
    }
}

/// The GPU side of a bricked volume, when there is a GPU.
pub(super) type Gpu<'a> = Option<(
    &'a crate::plots::volume::bricks::BrickTextures,
    &'a wgpu::Queue,
)>;
