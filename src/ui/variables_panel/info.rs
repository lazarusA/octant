//! Variable metadata overview card rendering.

use crate::data::VariableInfo;
use crate::i18n::{VariablesKey, fill, tr};
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::stack_str;
use egui::{RichText, Ui};
use std::fmt::Display;

/// Stack buffer for one translated detail line.
type LineBuf = [u8; 1024];

/// A small label of `key`'s template filled with `args`.
fn detail(ui: &mut Ui, key: VariablesKey, args: &[&dyn Display]) {
    let mut buf: LineBuf = [0; 1024];
    ui.small(fill(&mut buf, tr(key), args));
}

/// Renders variable data type, shape, dimensions, time range, and zattrs metadata.
pub fn show_variable_info(ui: &mut Ui, var_info: &VariableInfo) {
    ui.horizontal(|ui| {
        let mut buf = [0; 64];
        let data_type = stack_str(&mut buf, format_args!("[{}]", var_info.data_type));
        ui.label(RichText::new(data_type).small().weak());
    });

    if let Some(group) = var_info.group_path() {
        ui.horizontal(|ui| {
            ui.small(tr(VariablesKey::InfoPath));
            ui.icon(Icon::Folder, IconSize::Xs);
            for (i, seg) in group.split('/').filter(|s| !s.is_empty()).enumerate() {
                if i > 0 {
                    ui.icon_toned(Icon::ChevronRight, IconSize::Xs, IconTone::Muted);
                }
                ui.small(seg);
            }
        });
    }
    if let Some(units) = &var_info.units {
        detail(ui, VariablesKey::InfoUnits, &[units]);
    }
    if let Some(long_name) = &var_info.long_name {
        detail(ui, VariablesKey::InfoDescription, &[long_name]);
    }

    ui.separator();
    let mut buf = [0; 256];
    let shape = stack_str(&mut buf, format_args!("{:?}", var_info.shape));
    detail(ui, VariablesKey::InfoShape, &[&shape]);
    let mut buf = [0; 384];
    let dims = stack_str(&mut buf, format_args!("{:?}", var_info.dimension_names));
    detail(ui, VariablesKey::InfoDimensions, &[&dims]);

    if let (Some(start), Some(end)) = (&var_info.time_coverage_start, &var_info.time_coverage_end) {
        let start_clean = start.split('T').next().unwrap_or(start);
        let end_clean = end.split('T').next().unwrap_or(end);
        detail(ui, VariablesKey::InfoTime, &[&start_clean, &end_clean]);
    }
    if let Some(res) = &var_info.temporal_resolution {
        detail(ui, VariablesKey::InfoResolution, &[res]);
    }

    let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
    let mut buf = [0; 32];
    let size = stack_str(
        &mut buf,
        format_args!("{}", crate::i18n::number::Num(format_args!("{size_mb:.2}"))),
    );
    detail(ui, VariablesKey::InfoSize, &[&size]);
    crs_line(ui, var_info);

    if !var_info.attributes.is_empty() {
        egui::CollapsingHeader::new(tr(VariablesKey::InfoAttributes))
            .id_salt("variable_info_attributes")
            .show(ui, |ui| {
                for (k, v) in &var_info.attributes {
                    // Attribute values can be long: kept whole, only while open.
                    ui.small(format!("{k}: {v}"));
                }
            });
    }
}

/// The variable's CRS: its name (a WKT's or GeoTIFF citation, else its EPSG
/// code), its EPSG code beside a name, and its unit once resolved.
fn crs_line(ui: &mut Ui, var_info: &VariableInfo) {
    use crate::data::crs::{CrsRef, Resolution, named, resolve};
    let Some(crs) = &var_info.crs else {
        return;
    };
    let mut code_buf = [0; 24];
    let code = match crs {
        CrsRef::Epsg(code) => stack_str(&mut code_buf, format_args!("EPSG:{code}")),
        CrsRef::Geographic => "lon/lat",
        CrsRef::Wkt(_) => "WKT",
    };
    let mut buf = [0; 256];
    let text = match (named(var_info), crs) {
        (Some(name), CrsRef::Epsg(_)) => stack_str(&mut buf, format_args!("{name} ({code})")),
        (Some(name), _) => name,
        (None, _) => code,
    };
    let mut line = [0; 288];
    match resolve(crs) {
        Resolution::Ready(r) => {
            let text = stack_str(&mut line, format_args!("{text}, {}", r.unit));
            detail(ui, VariablesKey::InfoCrs, &[&text]);
        }
        Resolution::Pending => detail(ui, VariablesKey::InfoCrs, &[&text]),
        Resolution::Unsupported(_) => detail(ui, VariablesKey::InfoCrsUnsupported, &[&text]),
    }
}
