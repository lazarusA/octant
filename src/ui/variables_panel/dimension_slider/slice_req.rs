//! SliceRequest construction for plotted and selected dimension extents.

use crate::app::{OctantApp, VariableSelection};
use crate::data::slice_request::SliceRequest;

/// Builds a SliceRequest for currently plotted dimensions.
pub fn build_slice_request_for_plotted(
    app: &OctantApp,
    var_name: &str,
    shape: &[u64],
) -> SliceRequest {
    request(app, app.plotted(), app.channel_dim_index(), var_name, shape)
}

/// Builds a SliceRequest for currently selected dimensions.
pub fn build_slice_request(app: &OctantApp, var_name: &str, shape: &[u64]) -> SliceRequest {
    request(
        app,
        &app.selected,
        app.selected_channel_dim_index(),
        var_name,
        shape,
    )
}

/// `selection`'s request, reading the channel dimension whole while the
/// composite is on.
fn request(
    app: &OctantApp,
    selection: &VariableSelection,
    channel_dim: Option<usize>,
    var_name: &str,
    shape: &[u64],
) -> SliceRequest {
    let composite_dim = channel_dim.filter(|_| app.layers.base.composite.enabled);
    selection.slice_request(var_name, shape, composite_dim)
}
