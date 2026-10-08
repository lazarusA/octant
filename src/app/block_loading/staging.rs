//! Loads against the plotted selection while the UI stages another one (a
//! different variable or a plot still loading): playback keeps animating what
//! is shown until the staged plot's data arrives.

use crate::app::OctantApp;
use crate::app::layers::{LoadState, VariableSelection};
use crate::data::BlockCacheKey;
use std::mem::replace;

/// The staged selection and the bookkeeping of its pending request.
struct Staged {
    selection: VariableSelection,
    load: LoadState,
}

impl Staged {
    fn swap_with_plotted(app: &mut OctantApp) -> Self {
        Self {
            selection: replace(&mut app.selected, app.layers.base.selection().clone()),
            load: app.layers.base.load.clone(),
        }
    }

    fn restore(self, app: &mut OctantApp) {
        app.selected = self.selection;
        app.layers.base.load = self.load;
    }
}

impl OctantApp {
    /// Runs `f` with the plotted selection in place of the staged one: loads in
    /// `f` show (and re-sync) the plotted view, then the staged selection and
    /// its pending request come back untouched.
    pub(crate) fn with_plotted_selection(&mut self, f: impl FnOnce(&mut Self)) {
        let staged = Staged::swap_with_plotted(self);
        f(self);
        staged.restore(self);
    }

    /// Whether the UI stages something other than the plotted view: another
    /// variable, or a plot whose requested block is still loading.
    pub(crate) fn staging_differs(&self) -> bool {
        let request_pending = self
            .layers
            .base
            .load
            .block_key
            .as_ref()
            .is_some_and(|key| self.block_prefetcher.is_pending(key));
        request_pending || self.is_exploring_unplotted_variable()
    }

    /// Whether a block with cache key `key` belongs to the latest requested view
    /// (any step): blocks of an older selection, variable or plot layout must
    /// not be projected into it.
    pub(crate) fn key_matches_view(&self, key: &BlockCacheKey, anim_dim: Option<usize>) -> bool {
        self.layers
            .base
            .load
            .slice_request
            .as_ref()
            .is_some_and(|req| {
                key.variable_name == req.variable
                    && crate::data::blocks::key::selections_match_except_anim(
                        &key.selections,
                        &req.selections,
                        anim_dim,
                    )
            })
    }
}
