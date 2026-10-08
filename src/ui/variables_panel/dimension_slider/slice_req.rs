//! SliceRequest construction for plotted and selected dimension extents.

use crate::app::OctantApp;
use crate::data::slice_request::SliceRequest;

/// Builds a SliceRequest for currently plotted dimensions.
pub fn build_slice_request_for_plotted(
    app: &OctantApp,
    var_name: &str,
    shape: &[u64],
) -> SliceRequest {
    let composite_dim = app
        .channel_dim_index()
        .filter(|_| app.layers.base.composite.enabled);
    app.plotted().slice_request(var_name, shape, composite_dim)
}

/// Builds a SliceRequest for currently selected dimensions.
pub fn build_slice_request(app: &OctantApp, var_name: &str, shape: &[u64]) -> SliceRequest {
    let composite_dim = app
        .selected_channel_dim_index()
        .filter(|_| app.layers.base.composite.enabled);
    app.selected.slice_request(var_name, shape, composite_dim)
}
