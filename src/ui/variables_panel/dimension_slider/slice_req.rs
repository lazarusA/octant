//! SliceRequest construction for plotted and selected dimension extents.

use crate::app::OctantApp;
use crate::data::slice_request::{DimensionSelection, SliceRequest};

/// Builds a SliceRequest for currently plotted dimensions.
pub fn build_slice_request_for_plotted(
    app: &OctantApp,
    var_name: &str,
    shape: &[u64],
) -> SliceRequest {
    build_slice_request_from_ranges(
        var_name,
        shape,
        app.rgb_composite_mode,
        app.channel_dim_index(),
        &app.plotted_selected_dim_ranges,
    )
}

/// Builds a SliceRequest for currently selected dimensions.
pub fn build_slice_request(app: &OctantApp, var_name: &str, shape: &[u64]) -> SliceRequest {
    build_slice_request_from_ranges(
        var_name,
        shape,
        app.rgb_composite_mode,
        app.selected_channel_dim_index(),
        &app.selected_dim_ranges,
    )
}

fn build_slice_request_from_ranges(
    var_name: &str,
    shape: &[u64],
    rgb_composite: bool,
    c_dim: Option<usize>,
    ranges: &[(usize, usize)],
) -> SliceRequest {
    let selections = shape
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let dim_size = s as usize;
            if rgb_composite && c_dim == Some(i) {
                return DimensionSelection::Range {
                    start: 0,
                    end: dim_size,
                };
            }
            let (mut start, mut end) = ranges
                .get(i)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            if start == end {
                DimensionSelection::Index(start)
            } else {
                DimensionSelection::Range {
                    start,
                    end: (end + 1).min(dim_size),
                }
            }
        })
        .collect();

    SliceRequest {
        variable: var_name.to_string(),
        selections,
    }
}
