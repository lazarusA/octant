//! Step navigation controls and extent calculation.

use crate::app::OctantApp;

impl OctantApp {
    /// Requests the previous step along the animated dimension.
    pub fn step_prev(&mut self) {
        self.step_around(-1);
    }

    /// Requests the next step along the animated dimension.
    pub fn step_next(&mut self) {
        self.step_around(1);
    }

    /// Requests the step `by` steps away along the animated dimension,
    /// around the series (after the last step comes the first).
    pub fn step_around(&mut self, by: i64) {
        let steps = self.animated_dim_extent() as i64;
        if steps > 0 {
            let at = self.playback.current_timestep as i64;
            self.request_step_or_load((at + by).rem_euclid(steps) as usize);
        }
    }

    /// Requests the first step along the animated dimension.
    pub fn step_first(&mut self) {
        self.request_step_or_load(0);
    }

    /// Requests the last step along the animated dimension.
    pub fn step_last(&mut self) {
        let last = self.animated_dim_extent().saturating_sub(1);
        self.request_step_or_load(last);
    }

    /// One playback step, called by the frame timer: moves to the next step
    /// when its block is resident, and keeps the lookahead chunks queued.
    /// While another variable is only selected, or a new plot's block is still
    /// loading, it animates the plotted view (`with_plotted_selection`); the
    /// new plot replaces it when its block arrives.
    pub fn advance_playback(&mut self, now: web_time::Instant) {
        let total_extent = self.animated_dim_extent();
        if total_extent <= 1 {
            self.playback.is_playing = false;
            return;
        }
        let next_ts = if self.playback.current_timestep + 1 < total_extent {
            self.playback.current_timestep + 1
        } else if self.playback.loop_playback {
            0
        } else {
            self.playback.is_playing = false;
            return;
        };
        self.playback.last_step_time = now;
        if self.staging_differs() {
            self.with_plotted_selection(|app| app.play_step(next_ts));
        } else {
            self.play_step(next_ts);
        }
    }

    /// Shows step `next_ts` when every drawn animated layer holds it, else
    /// starts buffering; while buffering, waits until the buffer ahead of it
    /// is full (`may_show`). Then queues every layer's lookahead.
    fn play_step(&mut self, next_ts: usize) {
        if self.may_show(next_ts) {
            self.playback.buffering = false;
            self.playback.current_timestep = next_ts;
            self.load_step_blocks();
        } else {
            self.playback.buffering = true;
        }
        self.prefetch_animated_ranges();
    }

    /// Full size of the currently animated dimension in the dataset.
    pub fn animated_dim_extent(&self) -> usize {
        self.layer_animated_dim_extent(crate::app::layers::LayerId::BASE)
    }

    /// Full size of layer `id`'s animated dimension, or 1 without one.
    pub fn layer_animated_dim_extent(&self, id: crate::app::layers::LayerId) -> usize {
        let Some(anim_dim) = self.layers.get(id).and_then(|l| l.selection().animated_dim) else {
            return 1;
        };
        self.layer_variable_info(id)
            .and_then(|v| v.shape.get(anim_dim))
            .copied()
            .unwrap_or(1) as usize
    }

    /// Returns true if an animated dimension is currently selected and plotted.
    pub fn has_animated_dimension(&self) -> bool {
        self.plotted().animated_dim.is_some()
    }
}
