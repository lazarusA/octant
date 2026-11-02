//! What a layer's heatmap uploads: its data, or a pyramid's sample of the
//! view when resampled.

use crate::app::layers::Layer;
use crate::data::MatrixData;
use crate::plots::limits::max_storage_elements;
use std::sync::Arc;

/// The data uploaded to the GPU: `data` itself, or with `pyramid` (the
/// aggregation, when resampling) the pyramid's sample of the heatmap view
/// in `nav`, keeping the pyramid on the layer. Composites and single rows are
/// never resampled.
pub(super) fn shown_data(
    layer: &mut Layer,
    data: &MatrixData,
    pyramid: Option<crate::data::AggregationOp>,
    nav: &crate::app::state::NavigationState,
) -> MatrixData {
    let is_composite = layer.composite.enabled
        || data.dataset_name.contains("RGB Composite")
        || data.dataset_name.contains("CMYK Composite");
    let Some(op) = pyramid.filter(|_| !is_composite && data.height > 1) else {
        layer.data.pyramid = None;
        layer.data.resampler.set_pyramid(None);
        return data.clone();
    };
    let op = layer.pyramid_op(op);
    let pyramid = Arc::new(crate::data::MatrixPyramid::new(
        &data.values,
        data.width,
        data.height,
        &data.dataset_name,
        op,
        512,
    ));
    layer.data.resampler.set_pyramid(Some(pyramid.clone()));
    layer.data.pyramid = Some(pyramid.clone());
    let pan = [nav.heatmap_pan.x, nav.heatmap_pan.y];
    let (u, v) = crate::data::ViewportResampler::compute_visible_data_bounds(
        pan,
        nav.heatmap_zoom,
        [1.0, 1.0],
    );
    let target =
        crate::data::ViewportResampler::compute_target_resolution(data.width, data.height, 2048);
    pyramid.sample_viewport(&data.values, u, v, target)
}

/// Whether a layer of `cells` 2D cells is resampled through a pyramid: when
/// Resampling is ticked (`enabled`), or when it does not fit one GPU storage
/// buffer (`limits::max_storage_elements`, the device's own limit).
pub(crate) fn needs_pyramid(enabled: bool, cells: usize) -> bool {
    enabled || cells > max_storage_elements()
}
