//! Step navigation controls and extent calculation.

use crate::app::OctantApp;

impl OctantApp {
    /// Requests the previous step along the animated dimension.
    pub fn step_prev(&mut self) {
        let max_steps = self.animated_dim_extent();
        if max_steps > 0 {
            let prev_step = if self.playback.current_timestep > 0 {
                self.playback.current_timestep - 1
            } else {
                max_steps - 1
            };
            self.request_step_or_load(prev_step);
        }
    }

    /// Requests the next step along the animated dimension.
    pub fn step_next(&mut self) {
        let max_steps = self.animated_dim_extent();
        if max_steps > 0 {
            let next_step = (self.playback.current_timestep + 1) % max_steps;
            self.request_step_or_load(next_step);
        }
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

    /// Shows step `next_ts` when the base layer holds it (with the animated
    /// overlays), then queues every layer's lookahead.
    fn play_step(&mut self, next_ts: usize) {
        if self.layer_step_resident(crate::app::layers::LayerId::BASE, next_ts) {
            self.playback.current_timestep = next_ts;
            self.load_step_blocks();
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
