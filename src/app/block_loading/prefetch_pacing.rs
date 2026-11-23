//! How many lookahead fetches run at once and how many chunks they queue.

use super::pacing::{adaptive_concurrency, playing_lookahead, range_budget};
use crate::app::OctantApp;
use crate::app::layers::LayerId;

impl OctantApp {
    /// The lookahead's workers and its chunk budget for chunks of `cs` steps
    /// and `chunk_bytes`: the playing lookahead (the buffer's steps at least,
    /// the animated layers sharing the cache), else the paused range's.
    /// Foreground requests (each layer's shown block) never take the
    /// lookahead's workers. Stored chunks of `stored_bytes` past
    /// `LARGE_CHUNK_BYTES` pace by that size (every fetch decodes a whole
    /// one), and take one worker in the browser: its download budget holds
    /// one such chunk and its decoded copy at a time.
    pub(super) fn prefetch_pacing(
        &self,
        id: LayerId,
        cs: usize,
        chunk_bytes: usize,
        stored_bytes: usize,
    ) -> (usize, usize) {
        let large = stored_bytes > super::arrival_notices::LARGE_CHUNK_BYTES;
        let lookahead = match (large, cfg!(target_arch = "wasm32")) {
            (true, true) => 1,
            // Several whole chunks decode at once: pace by their size.
            (true, false) => adaptive_concurrency(stored_bytes, self.prefetch_threads),
            (false, _) => adaptive_concurrency(chunk_bytes, self.prefetch_threads),
        };
        let max_concurrent = lookahead + self.foreground_requests();
        let budget = if self.playback.is_playing {
            self.playing_lookahead_chunks(cs, chunk_bytes, self.ring_lead(id))
        } else {
            range_budget(chunk_bytes, self.block_cache.max_bytes())
        };
        (max_concurrent, budget)
    }

    /// Chunks of `cs` steps (`chunk_bytes` each) one layer queues while
    /// playing, the current one included: it and the buffer's steps after it
    /// (the buffer counts from the next step) at least, within the layer's
    /// share of the cache, and `lead` more for a ring volume's window. The
    /// buffer waits for no more than this holds.
    pub(crate) fn playing_lookahead_chunks(
        &self,
        cs: usize,
        chunk_bytes: usize,
        lead: usize,
    ) -> usize {
        let window = self.block_window_size.max(self.buffer_target_steps() + cs) + lead;
        let share = self.block_cache.max_bytes() / self.animated_layer_count();
        playing_lookahead(window, cs, chunk_bytes, share)
    }

    /// Blocks being fetched to be shown: layers' and detail tiles'.
    fn foreground_requests(&self) -> usize {
        let pending = |k: Option<&crate::data::BlockCacheKey>| {
            usize::from(k.is_some_and(|k| self.block_prefetcher.is_pending(k)))
        };
        self.layers
            .iter()
            .map(|l| {
                pending(l.load.block_key.as_ref())
                    + pending(l.data.detail.pending.as_ref().map(|p| &p.key))
            })
            .sum()
    }
}
