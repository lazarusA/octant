//! Multi-channel composite default initialization and OMERO attribute extraction.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::VariableInfo;

/// Initialize the base layer's composite defaults for a variable.
pub fn init_composite_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    init_layer_composite_defaults(app, LayerId::BASE, var_info);
}

/// Initializes layer `id`'s composite for `var_info`: channel configs for a
/// channel dimension with only the selected channel ticked, the composite on
/// only for a GeoTIFF that opens in color ([`opens_in_color`]).
pub fn init_layer_composite_defaults(app: &mut OctantApp, id: LayerId, var_info: &VariableInfo) {
    let is_tiff = app.layer_is_geotiff(id);
    let channel = var_info.channel_axis();
    let ticked = channel.map(|c| app.layer_dim_range(id, c).0);
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
    composite.enabled = opens_in_color(var_info, is_tiff);
    composite.channel_configs = configs;
}

/// Whether a GeoTIFF raster opens as a color composite: 3 bands (RGB), 4
/// whose last is alpha, or CMYK inks. Every other variable opens on one band.
pub(crate) fn opens_in_color(var_info: &VariableInfo, is_tiff: bool) -> bool {
    let attr = |key: &str| var_info.attributes.get(key).map(String::as_str);
    let bands = var_info.shape.first().copied().unwrap_or(0);
    is_tiff
        && var_info.shape.len() >= 3
        && match bands {
            3 => true,
            4 if attr("alpha_band") == Some("last") => true,
            _ => bands >= 4 && crate::data::slicing::composite::cmyk::is_cmyk_var(var_info),
        }
}

/// One config per channel, from the OME-Zarr `omero` channels when there are
/// any, else default colors; only channel `ticked` is visible.
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
                    visible: true,
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
            cfg.visible = Some(i) == ticked;
            cfg
        })
        .collect()
}
