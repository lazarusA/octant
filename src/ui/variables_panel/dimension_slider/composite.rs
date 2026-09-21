//! Multi-channel composite default initialization and OMERO attribute extraction.

use crate::app::OctantApp;
use crate::data::VariableInfo;

/// Initialize multi-channel composite configurations and defaults for a variable.
pub fn init_composite_defaults(app: &mut OctantApp, var_info: &VariableInfo, rank: usize) {
    app.rgb_composite_channels = [0, 1, 2];
    app.composite_channel_configs.clear();

    let has_omero = var_info.attributes.contains_key("omero_channels")
        || var_info.attributes.contains_key("omero_colors");

    let c_idx_opt = var_info
        .dimension_names
        .iter()
        .position(|d| crate::data::coordinates::naming::is_channel_dim_name(d))
        .or(if has_omero && rank >= 3 {
            Some(0)
        } else {
            None
        });

    if let Some(c_idx) = c_idx_opt {
        let num_ch = var_info.shape.get(c_idx).copied().unwrap_or(0) as usize;
        if num_ch >= 2 {
            app.composite_channel_configs = extract_channel_configs(var_info, num_ch);
        }
    }

    if has_omero && !app.composite_channel_configs.is_empty() {
        app.rgb_composite_mode = true;
        app.active_colormap = 1000;
    } else if rank < 3 || var_info.shape.first().copied().unwrap_or(0) < 3 {
        app.rgb_composite_mode = false;
        if app.active_colormap == 1000 {
            app.active_colormap = 0;
        }
    }
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
