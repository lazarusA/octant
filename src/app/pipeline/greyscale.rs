//! The display of greyscale OME-Zarr images (OMERO `model: "greyscale"`).

/// A greyscale OME-Zarr image (`model: "greyscale"`) shows its channel in
/// gray, through the channel's window (locked) and inversion.
pub(super) fn apply_greyscale_display(layer: &mut crate::app::layers::Layer) {
    let display = {
        let sel = layer.selection();
        sel.metadata
            .as_ref()
            .and_then(|m| m.variables.get(sel.variable_idx))
            .and_then(|v| {
                let image = &v.ome.as_ref()?.image;
                let c = image.axis_of_kind(crate::data::ome::AxisKind::Channel)?;
                crate::data::ome::model::greyscale_display(image, *sel.dim_indices.get(c)?)
            })
    };
    let Some((window, inverted)) = display else {
        return;
    };
    let color = &mut layer.color;
    color.colormap = crate::utils::colormap::registry::find("classic:gray")
        .unwrap_or_else(crate::utils::colormap::registry::default_id);
    color.reversed = inverted;
    if let Some(w) = window.filter(|w| w.end > w.start) {
        color.range_min = w.start;
        color.range_max = w.end;
        color.lock_bounds = true;
    }
}
