//! The display of greyscale OME-Zarr images (OMERO `model: "greyscale"`).

/// A greyscale OME-Zarr image (`model: "greyscale"`) shows its channel in
/// gray, through the channel's window (locked) and inversion, set again
/// whenever another channel is shown.
/// `step` is the layer's step, the channel shown when the channel axis is
/// the animated one; `data_range` is the shown data's, for channels without
/// a window.
pub(crate) fn apply_greyscale_display(
    layer: &mut crate::app::layers::Layer,
    step: usize,
    data_range: (f32, f32),
) {
    let display = {
        let sel = layer.selection();
        sel.variable_info().and_then(|v| {
            let image = &v.ome.as_ref()?.image;
            let c = image.axis_of_kind(crate::data::ome::AxisKind::Channel)?;
            let channel = if sel.animated_dim() == Some(c) {
                step
            } else {
                sel.dim_config.get(c)?.index
            };
            let display = crate::data::ome::model::greyscale_display(image, channel)?;
            Some((channel, display))
        })
    };
    let Some((channel, (window, inverted))) = display else {
        return;
    };
    if layer.data.greyscale_channel == Some(channel) {
        return;
    }
    layer.data.greyscale_channel = Some(channel);
    let color = &mut layer.color;
    color.colormap = crate::utils::colormap::registry::find("classic:gray")
        .unwrap_or_else(crate::utils::colormap::registry::default_id);
    color.reversed = inverted;
    match window.filter(|w| w.end > w.start) {
        Some(w) => color.set_range(w.start, w.end, true),
        None => color.set_range(data_range.0, data_range.1, false),
    }
}
