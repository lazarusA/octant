//! Plot type switches adapt each selection by its own variable.

use crate::app::test_support::{memory_app, poll_until, selection_of};
use crate::plots::PlotType;

/// A composite volume plotted, then a 2D variable staged: leaving 3D must not
/// put the plotted variable's band range on the staged variable's rows.
#[test]
fn leaving_3d_keeps_a_staged_2d_variable_whole() {
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "rgb");
    app.switch_plot_type(PlotType::Volume);
    app.plot_selection();
    poll_until(&mut app, |app| app.layers.base.data.volume.is_some());
    app.layers.base.composite.enabled = true;

    app.selected = selection_of(&mut app, &meta, "elev");
    let staged = app.selected.dim_ranges.clone();
    app.switch_plot_type(PlotType::Surface);
    assert_eq!(app.selected.dim_ranges, staged);
    // The plotted composite keeps every band.
    assert_eq!(app.plotted().dim_ranges[0], (0, 2));
}
