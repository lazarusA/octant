//! What a selection of a multiscale variable reads, for its sizes and GPU
//! limits: the level its plot reads and the level-0 ranges on that level's
//! cells, while the UI stays on level 0.

use std::borrow::Cow;

use crate::app::OctantApp;
use crate::app::layers::VariableSelection;
use crate::app::multiscale::{level_range, variable_index};
use crate::data::VariableInfo;

/// The variable a selection reads and its inclusive ranges on that
/// variable's cells (`OctantApp::read_window`).
pub(crate) type ReadWindow<'a> = (&'a VariableInfo, Cow<'a, [(usize, usize)]>);

impl OctantApp {
    /// What `selection` reads, for its sizes and GPU limits: the variable of
    /// the level its plot reads and its ranges on that level's cells (its
    /// own variable and ranges unless it is a stack's level 0). The UI stays
    /// on level 0; nothing but the ranges is copied.
    pub(crate) fn read_window<'a>(
        &self,
        selection: &'a VariableSelection,
    ) -> Option<ReadWindow<'a>> {
        self.read_window_as(selection, selection.plot_type)
    }

    /// [`Self::read_window`] as plot type `plot_type` would read it, for the
    /// sizes of plot types not picked yet.
    pub(crate) fn read_window_as<'a>(
        &self,
        selection: &'a VariableSelection,
        plot_type: crate::plots::PlotType,
    ) -> Option<ReadWindow<'a>> {
        let var = selection.variable_info()?;
        let own = Some((var, Cow::Borrowed(selection.dim_ranges.as_slice())));
        let Some(level) = self
            .selection_level_as(selection, plot_type)
            .filter(|&l| l > 0)
        else {
            return own;
        };
        let (Some(set), Some(meta)) = (var.stack_base().map(|b| &b.set), &selection.metadata)
        else {
            return own;
        };
        let read = set
            .level(level)
            .and_then(|l| variable_index(meta, &l.path))
            .and_then(|i| meta.variables.get(i));
        let Some(read) = read else {
            return own;
        };
        let ranges = (selection.dim_ranges.iter().enumerate())
            .map(|(d, &r)| level_range(set, level, d, r))
            .collect();
        Some((read, Cow::Owned(ranges)))
    }
}
