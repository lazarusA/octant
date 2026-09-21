//! Step navigation controls and extent calculation.

use crate::app::OctantApp;

impl OctantApp {
    /// Requests the previous step along the animated dimension.
    pub fn step_prev(&mut self) {
        let max_steps = self.animated_dim_extent();
        if max_steps > 0 {
            let prev_step = if self.current_timestep > 0 {
                self.current_timestep - 1
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
            let next_step = (self.current_timestep + 1) % max_steps;
            self.request_step_or_load(next_step);
        }
    }

    /// Full size of the currently animated dimension in the dataset.
    pub fn animated_dim_extent(&self) -> usize {
        let Some(anim_dim) = self.plotted_animated_dim else {
            return 1;
        };
        self.plotted_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.plotted_variable_idx))
            .and_then(|v| v.shape.get(anim_dim))
            .map(|&s| s as usize)
            .unwrap_or(1)
    }
}
