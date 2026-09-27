//! SliceRequest construction for plotted and selected dimension extents.

use crate::app::OctantApp;
use crate::data::slice_request::{DimensionSelection, SliceRequest};

/// Builds a SliceRequest for currently plotted dimensions.
pub fn build_slice_request_for_plotted(
    app: &OctantApp,
    var_name: &str,
    shape: &[u64],
) -> SliceRequest {
    let c_dim = app.channel_dim_index();
    let selections = shape
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let dim_size = s as usize;
            if app.rgb_composite_mode && c_dim == Some(i) {
                return DimensionSelection::Range {
                    start: 0,
                    end: dim_size,
                };
            }
            let (start, end) = app
                .plotted_selected_dim_ranges
                .get(i)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
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

/// Builds a SliceRequest for currently selected dimensions.
pub fn build_slice_request(app: &OctantApp, var_name: &str, shape: &[u64]) -> SliceRequest {
    let c_dim = app.selected_channel_dim_index();
    let selections = shape
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let dim_size = s as usize;
            if app.rgb_composite_mode && c_dim == Some(i) {
                return DimensionSelection::Range {
                    start: 0,
                    end: dim_size,
                };
            }
            let (start, end) = app
                .selected_dim_ranges
                .get(i)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
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
