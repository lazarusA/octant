//! Dimension row controls and metric banners.

use crate::app::{AnimationRole, OctantApp};
use crate::data::VariableInfo;
use crate::i18n::{VariablesKey as K, fill, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::ui::number_input::LocalizedInput;
use crate::utils::{ByteSize, format_count_metric, stack_str};
use egui::{RichText, Ui};
use std::fmt::Display;

use super::coord_label::show_coord_label;
use super::double_slider::double_slider_with_inputs;
use super::metrics::{
    calculate_download_sizes, calculate_selected_2d_elements, calculate_selected_volume_elements,
};
use super::role_row::{DimName, role_row};
use super::roles::apply_role_change;
use crate::ui::flow_row::{self, VALUE_BOX_W};

/// Renders the complete dimension sliders section including capacity and bandwidth metrics.
pub fn show_dimension_sliders(app: &mut OctantApp, ui: &mut Ui, var_info: &VariableInfo) {
    let rank = var_info.shape.len();

    if app.selected.dim_config.len() != rank {
        super::defaults::init_variable_dimension_defaults(app, var_info);
    }

    let (requested_bytes, total_bytes) =
        calculate_download_sizes(var_info, &app.selected.dim_config, &app.selected.dim_ranges);

    let requested_cells: u64 = if rank > 0 {
        var_info
            .shape
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                if app.selected.dim_config.get(i).is_some_and(|c| c.active) {
                    if let Some(&(start, end)) = app.selected.dim_ranges.get(i) {
                        (end.saturating_sub(start) + 1).min(s as usize) as u64
                    } else {
                        s
                    }
                } else {
                    1
                }
            })
            .try_fold(1u64, |acc, x| acc.checked_mul(x))
            .unwrap_or(u64::MAX)
    } else {
        1
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(tr(K::Download)).strong());
        let cells = format_count_metric(requested_cells.min(usize::MAX as u64) as usize);
        let mut buf = [0; 128];
        let requested = ByteSize(requested_bytes);
        let text = fill(&mut buf, tr(K::DownloadCells), &[&requested, &cells]);
        ui.label(RichText::new(text).strong());
        let mut buf = [0; 32];
        let total = stack_str(&mut buf, format_args!("/ {}", ByteSize(total_bytes)));
        ui.label(RichText::new(total).weak());
    });
    ui.add_space(4.0);

    let total_2d_elements = calculate_selected_2d_elements(app);
    let large_2d = if total_2d_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        Some((Icon::Bolt, IconTone::Info, K::LargeSelectionPyramid))
    } else if total_2d_elements > crate::plots::common::MAX_2D_SURFACE_ELEMENTS {
        Some((Icon::Info, IconTone::Warning, K::LargeSelectionNoMeshes))
    } else {
        None
    };
    if let Some((icon, tone, key)) = large_2d {
        let cells = format_count_metric(total_2d_elements);
        let mut buf = [0; 24];
        let data_mb = megabytes(&mut buf, total_2d_elements);
        banner(ui, icon, tone, key, &[&cells, &data_mb]);
    }

    let total_vol_elements = calculate_selected_volume_elements(app);
    if total_vol_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        let mut buf = [0; 24];
        let vol_mb = megabytes(&mut buf, total_vol_elements);
        let key = K::LargeVolumeDisabled;
        banner(ui, Icon::Warning, IconTone::Warning, key, &[&vol_mb]);
    }

    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("dim");

        let is_animated = app.selected.dim_config[i].animation == AnimationRole::Animated;

        ui.group(|ui| {
            // Fill the panel width so every dimension box lines up.
            ui.set_width(ui.available_width());
            let mut spatial = app.selected.dim_config[i].spatial;
            let mut anim = app.selected.dim_config[i].animation;
            let name = DimName {
                name: dim_name,
                size: dim_size,
            };
            let active = &mut app.selected.dim_config[i].active;
            role_row(ui, i, name, active, &mut spatial, &mut anim);
            apply_role_change(i, spatial, anim, app);

            ui.add_space(4.0);

            if app.selected.dim_config[i].active {
                let (mut start, mut end) = app.selected.dim_ranges[i];
                double_slider_with_inputs(
                    ui,
                    dim_name,
                    &mut start,
                    &mut end,
                    0,
                    dim_size.saturating_sub(1),
                );

                app.selected.dim_ranges[i] = (start, end);
                if is_animated {
                    let max_idx = dim_size.saturating_sub(1);
                    if app.playback.is_playing
                        || app.playback.current_timestep > end
                        || app.playback.current_timestep < start
                    {
                        app.selected.dim_indices[i] = app.playback.current_timestep.min(max_idx);
                    } else {
                        app.selected.dim_indices[i] =
                            app.playback.current_timestep.clamp(start, end);
                    }
                } else {
                    app.selected.dim_indices[i] = start;
                }
                app.selected.dim_config[i].range = (start, end);
                app.selected.dim_config[i].index = app.selected.dim_indices[i];
                show_coord_label(app, ui, var_info, i, (start, end));
            } else {
                let max_index = dim_size.saturating_sub(1);
                if is_animated
                    && (app.playback.is_playing
                        || app.playback.current_timestep != app.selected.dim_indices[i])
                {
                    app.selected.dim_indices[i] = app.playback.current_timestep.min(max_index);
                }
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label(tr(K::Index));
                    // Slider fills the row up to a fixed-width value box,
                    // matching the range rows. The slider runs first so the box
                    // shows the dragged value on the same frame.
                    let index = &mut app.selected.dim_indices[i];
                    ui.spacing_mut().slider_width = flow_row::track_width(ui, VALUE_BOX_W);
                    let s_resp = ui.add(
                        egui::Slider::new(index, 0..=max_index)
                            .localized()
                            .show_value(false),
                    );
                    let height = ui.spacing().interact_size.y;
                    let v_resp = ui.add_sized(
                        [VALUE_BOX_W, height],
                        egui::DragValue::new(index)
                            .localized()
                            .range(0..=max_index)
                            .speed(1),
                    );
                    changed = s_resp.dragged() || v_resp.dragged() || s_resp.clicked();
                });
                if changed && is_animated {
                    app.playback.current_timestep = app.selected.dim_indices[i];
                }
                app.selected.dim_ranges[i] =
                    (app.selected.dim_indices[i], app.selected.dim_indices[i]);
                app.selected.dim_config[i].range = app.selected.dim_ranges[i];
                app.selected.dim_config[i].index = app.selected.dim_indices[i];
                show_coord_label(app, ui, var_info, i, app.selected.dim_ranges[i]);
            }
        });

        ui.add_space(6.0);
    }
}

/// Megabytes taken by `elements` `f32` values, without decimals.
fn megabytes(buf: &mut [u8], elements: usize) -> &str {
    let mb = (elements as f64 * 4.0) / (1024.0 * 1024.0);
    stack_str(buf, format_args!("{mb:.0}"))
}

/// A grouped, wrapped notice: `icon` and `key`'s template filled with `args`,
/// small and in `tone`'s color.
fn banner(ui: &mut Ui, icon: Icon, tone: IconTone, key: K, args: &[&dyn Display]) {
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.icon_toned(icon, IconSize::Xs, tone);
            let mut buf = [0; 768];
            let text = fill(&mut buf, tr(key), args);
            ui.label(RichText::new(text).small().color(tone.color(ui.visuals())));
        });
    });
    ui.add_space(2.0);
}
