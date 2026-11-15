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

/// Renders variable data type, shape, dimensions, time range, and the button
/// that opens its attributes.
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
    attributes_button(ui, var_info);
}

/// Tallest the attributes popup gets before it scrolls.
const ATTRIBUTES_MAX_H: f32 = 420.0;

/// "Attributes" and their count; a click opens them in a popup as wide as
/// the panel, so long values (history, WKT) never stretch the panel.
fn attributes_button(ui: &mut Ui, var_info: &VariableInfo) {
    if var_info.attributes.is_empty() {
        return;
    }
    let width = ui.available_width();
    let button = ui
        .horizontal(|ui| {
            let button = ui.outlined_icon_button(
                Icon::Info,
                tr(VariablesKey::InfoAttributes),
                IconTone::Default,
            );
            let mut buf = [0; 24];
            let count = stack_str(&mut buf, format_args!("{}", var_info.attributes.len()));
            ui.label(RichText::new(count).small().weak());
            button
        })
        .inner;
    show_attributes(
        egui::Popup::from_toggle_button_response(&button),
        width,
        var_info,
    );
}

/// Shows `var_info`'s attributes in `popup`, `width` wide, scrolling past
/// `ATTRIBUTES_MAX_H`; closes on a click outside. Shared by the Dimensions
/// panel's info card and the colorbar's info button.
pub fn show_attributes(popup: egui::Popup<'_>, width: f32, var_info: &VariableInfo) {
    // No frame margin: the scroll area spans the popup, so its bar sits on the
    // popup's right edge; the rows get their padding inside it instead.
    let frame = egui::Frame::popup(&popup.ctx().global_style()).inner_margin(0.0);
    popup
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .frame(frame)
        .width(width)
        .show(|ui| {
            egui::ScrollArea::vertical()
                .id_salt("variable_info_attributes")
                .auto_shrink([false, true])
                .max_height(ATTRIBUTES_MAX_H)
                .show(ui, |ui| attribute_rows(ui, var_info));
        });
}

/// Padding around the attribute rows; the right side leaves room for the
/// scroll bar, which floats over the content.
const ROWS_MARGIN: egui::Margin = egui::Margin {
    left: 12,
    right: 18,
    top: 10,
    bottom: 10,
};

/// One attribute per row: its key, then its value wrapped (selectable to copy).
fn attribute_rows(ui: &mut Ui, var_info: &VariableInfo) {
    egui::Frame::NONE.inner_margin(ROWS_MARGIN).show(ui, |ui| {
        for (key, value) in &var_info.attributes {
            ui.label(RichText::new(key.as_str()).strong());
            ui.label(RichText::new(value.as_str()));
            ui.add_space(4.0);
        }
    });
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
        CrsRef::Proj4(proj4) => (proj4.split_whitespace())
            .find_map(|t| t.strip_prefix("+proj="))
            .unwrap_or("proj4"),
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
