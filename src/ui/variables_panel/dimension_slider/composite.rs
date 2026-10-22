//! Multi-channel composite default initialization and OMERO attribute extraction.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::VariableInfo;

/// Initialize the base layer's composite defaults for a variable.
pub fn init_composite_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    init_layer_composite_defaults(app, LayerId::BASE, var_info);
}

/// Initializes layer `id`'s composite for `var_info`: channel configs for a
/// channel dimension, the composite on for a variable that opens in color
/// ([`opens_in_color`]), with OMERO's active channels ticked, else off with
/// only the selected channel ticked.
pub fn init_layer_composite_defaults(app: &mut OctantApp, id: LayerId, var_info: &VariableInfo) {
    let is_tiff = app.layer_is_geotiff(id);
    let in_color = opens_in_color(var_info, is_tiff);
    let channel = var_info.channel_axis();
    let ticked = channel
        .filter(|_| !in_color)
        .map(|c| app.layer_dim_range(id, c).0);
    let configs = if is_tiff {
        Vec::new()
    } else {
        channel
            .map(|c| var_info.shape.get(c).copied().unwrap_or(0) as usize)
            .filter(|&channels| channels >= 2)
            .map(|channels| extract_channel_configs(var_info, channels, ticked))
            .unwrap_or_default()
    };
    let Some(layer) = app.layers.get_mut(id) else {
        return;
    };
    let composite = &mut layer.composite;
    composite.rgb_channels = [0, 1, 2];
    composite.enabled = in_color;
    composite.channel_configs = configs;
}

/// Whether a variable opens as a color composite: a GeoTIFF raster of 3 bands
/// (RGB), 4 whose last is alpha, or CMYK inks; an OME-Zarr image of 2+
/// channels that is not greyscale (its active channels mixed within the
/// channel range). Every other variable opens on one band.
pub(crate) fn opens_in_color(var_info: &VariableInfo, is_tiff: bool) -> bool {
    if !is_tiff {
        return ome_in_color(var_info);
    }
    let attr = |key: &str| var_info.attributes.get(key).map(String::as_str);
    let bands = var_info.shape.first().copied().unwrap_or(0);
    var_info.shape.len() >= 3
        && match bands {
            3 => true,
            4 if attr("alpha_band") == Some("last") => true,
            _ => bands >= 4 && crate::data::slicing::composite::cmyk::is_cmyk_var(var_info),
        }
}

/// Whether an OME-Zarr image mixes its channels: 2+ of them, not greyscale.
fn ome_in_color(var_info: &VariableInfo) -> bool {
    let channels = var_info.channel_axis().and_then(|c| var_info.shape.get(c));
    var_info
        .ome
        .as_ref()
        .is_some_and(|o| o.image.greyscale_channel().is_none())
        && channels.is_some_and(|&n| n >= 2)
}

/// One config per channel, from the OME-Zarr `omero` channels when there are
/// any, else default colors; only channel `ticked` is visible, or without one
/// OMERO's active channels.
fn extract_channel_configs(
    var_info: &VariableInfo,
    num_ch: usize,
    ticked: Option<usize>,
) -> Vec<crate::data::slicing::ChannelColorConfig> {
    let channels = var_info
        .ome
        .as_ref()
        .map_or(&[][..], |o| o.image.channels.as_slice());
    (0..num_ch)
        .map(|i| {
            let mut cfg = match channels.get(i) {
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
            };
            if let Some(t) = ticked {
                cfg.visible = i == t;
            }
            cfg
        })
        .collect()
}
