//! Multi-channel composite default initialization and OMERO attribute extraction.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::VariableInfo;

/// Initialize the base layer's composite defaults for a variable.
pub fn init_composite_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    init_layer_composite_defaults(app, LayerId::BASE, var_info);
}

/// Initializes layer `id`'s composite for `var_info`: OMERO channel configs
/// for a channel dimension (multi-channel overlay), RGB on for a 3+ band
/// GeoTIFF, otherwise off.
pub fn init_layer_composite_defaults(app: &mut OctantApp, id: LayerId, var_info: &VariableInfo) {
    let rank = var_info.shape.len();
    let is_tiff = app.layer_is_geotiff(id);
    let has_omero = var_info.attributes.contains_key("omero_channels")
        || var_info.attributes.contains_key("omero_colors");
    let configs = if is_tiff {
        Vec::new()
    } else {
        var_info
            .dimension_names
            .iter()
            .position(|d| crate::data::coordinates::naming::is_channel_dim_name(d))
            .or((has_omero && rank >= 3).then_some(0))
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
    composite.enabled = (is_tiff && rank >= 3 && num_bands >= 3) || !configs.is_empty();
    composite.channel_configs = configs;
}

fn extract_channel_configs(
    var_info: &VariableInfo,
    num_ch: usize,
) -> Vec<crate::data::slicing::ChannelColorConfig> {
    let channel_labels: Vec<String> = var_info
        .attributes
        .get("omero_channels")
        .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();

    let channel_colors: Vec<String> = var_info
        .attributes
        .get("omero_colors")
        .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();

    let channel_windows: Vec<String> = var_info
        .attributes
        .get("omero_windows")
        .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();

    let channel_actives: Vec<String> = var_info
        .attributes
        .get("omero_actives")
        .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();

    let mut configs = Vec::with_capacity(num_ch);
    for i in 0..num_ch {
        let name = channel_labels
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("Channel {}", i + 1));

        let color_rgb = channel_colors
            .get(i)
            .and_then(|h| crate::data::slicing::parse_hex_color(h))
            .unwrap_or_else(|| {
                crate::data::slicing::DEFAULT_CHANNEL_COLORS
                    [i % crate::data::slicing::DEFAULT_CHANNEL_COLORS.len()]
            });

        let visible = channel_actives
            .get(i)
            .map(|a| a.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        let window = channel_windows.get(i).and_then(|w_str| {
            let mut parts = w_str.split(':');
            let start = parts.next()?.parse::<f32>().ok()?;
            let end = parts.next()?.parse::<f32>().ok()?;
            Some((start, end))
        });

        configs.push(crate::data::slicing::ChannelColorConfig {
            index: i,
            name,
            color_rgb,
            visible,
            window,
        });
    }
    configs
}
