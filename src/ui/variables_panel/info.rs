//! Variable metadata overview card rendering.

use crate::data::VariableInfo;
use crate::ui::icons::{Icon, UiIconExt};
use egui::{RichText, Ui};

/// Renders variable data type, shape, dimensions, time range, and zattrs metadata.
pub fn show_variable_info(ui: &mut Ui, var_info: &VariableInfo) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("[{}]", var_info.data_type))
                .small()
                .weak(),
        );
    });

    if let Some(group) = var_info.group_path() {
        ui.horizontal(|ui| {
            ui.small("Path:");
            ui.icon(Icon::Folder, 10.0);
            for (i, seg) in group.split('/').filter(|s| !s.is_empty()).enumerate() {
                if i > 0 {
                    ui.icon_colored(Icon::ChevronRight, 8.0, ui.visuals().weak_text_color());
                }
                ui.small(seg);
            }
        });
    }
    if let Some(units) = &var_info.units {
        ui.small(format!("Units: {}", units));
    }
    if let Some(long_name) = &var_info.long_name {
        ui.small(format!("Description: {}", long_name));
    }

    ui.separator();
    ui.small(format!("Shape: {:?}", var_info.shape));
    ui.small(format!("Dimensions: {:?}", var_info.dimension_names));

    if let (Some(start), Some(end)) = (&var_info.time_coverage_start, &var_info.time_coverage_end) {
        let start_clean = start.split('T').next().unwrap_or(start);
        let end_clean = end.split('T').next().unwrap_or(end);
        ui.small(format!("Time: {} -> {}", start_clean, end_clean));
    }
    if let Some(res) = &var_info.temporal_resolution {
        ui.small(format!("Resolution: {}", res));
    }

    let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
    ui.small(format!("Size: {:.2} MB", size_mb));

    if !var_info.attributes.is_empty() {
        ui.collapsing("Attributes (.zattrs)", |ui| {
            for (k, v) in &var_info.attributes {
                ui.small(format!("{}: {}", k, v));
            }
        });
    }
}
