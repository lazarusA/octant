//! Multi-channel composite default initialization and OMERO attribute extraction.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::VariableInfo;

/// Initialize the base layer's composite defaults for a variable.
pub fn init_composite_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    init_layer_composite_defaults(app, LayerId::BASE, var_info);
}

/// Initializes layer `id`'s composite for `var_info`: OME-Zarr channel
/// configs for a channel dimension (multi-channel overlay), RGB on for a 3+
/// band GeoTIFF, otherwise off.
pub fn init_layer_composite_defaults(app: &mut OctantApp, id: LayerId, var_info: &VariableInfo) {
    let rank = var_info.shape.len();
    let is_tiff = app.layer_is_geotiff(id);
    let configs = if is_tiff {
        Vec::new()
    } else {
        var_info
            .channel_axis()
            .map(|c| var_info.shape.get(c).copied().unwrap_or(0) as usize)
            .filter(|&channels| channels >= 2)
            .map(|channels| extract_channel_configs(var_info, channels))
            .unwrap_or_default()
    };
    let num_bands = var_info.shape.first().copied().unwrap_or(0) as usize;
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let composite = &mut layer.composite;
    composite.rgb_channels = [0, 1, 2];
    // A greyscale OME-Zarr image shows one channel; the overlay stays a toggle away.
    let greyscale = var_info
        .ome
        .as_ref()
        .is_some_and(|o| o.image.greyscale_channel().is_some());
    composite.enabled =
        (is_tiff && rank >= 3 && num_bands >= 3) || (!configs.is_empty() && !greyscale);
    composite.channel_configs = configs;
}

/// One config per channel, from the OME-Zarr `omero` channels when there are
/// any, else default colors.
fn extract_channel_configs(
    var_info: &VariableInfo,
    num_ch: usize,
) -> Vec<crate::data::slicing::ChannelColorConfig> {
    let channels = var_info
        .ome
        .as_ref()
        .map_or(&[][..], |o| o.image.channels.as_slice());
    (0..num_ch)
        .map(|i| match channels.get(i) {
            // Unnamed channels stay empty and are named in the UI language
            // when drawn ("Channel N").
            Some(ch) => crate::data::slicing::ChannelColorConfig {
                index: i,
                name: ch.label.clone(),
                color_rgb: ch.color,
                visible: ch.active,
                window: ch.window.map(|w| (w.start, w.end)),
                display: crate::data::slicing::ChannelDisplay {
                    range: ch.window.map(|w| (w.min, w.max)),
                    inverted: ch.inverted,
                    family: ch.family,
                    coefficient: ch.coefficient,
                },
            },
            None => crate::data::slicing::ChannelColorConfig::new(
                i,
                String::new(),
                crate::data::slicing::DEFAULT_CHANNEL_COLORS
                    [i % crate::data::slicing::DEFAULT_CHANNEL_COLORS.len()],
            ),
        })
        .collect()
}
