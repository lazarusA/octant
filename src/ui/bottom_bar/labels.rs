//! Text items of the bottom bar: the current-step date (hourglass icon plus
//! value, with a details popover) and the start, step-size and end badges.

use super::layout::BottomBarItem as Item;
use super::timeline::{Timeline, dim_name};
use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, UiIconExt};
use crate::utils::stack_str;

/// Stack buffer for badge labels.
const LABEL_BUF_LEN: usize = 160;
type LabelBuf = [u8; LABEL_BUF_LEN];

/// Text shown by a text item: the current value or a bracketed badge. The
/// start value is not repeated here; the start badge shows it.
fn text<'a>(buf: &'a mut LabelBuf, item: Item, tl: &'a Timeline) -> &'a str {
    match item {
        Item::DateInfo => &tl.current,
        Item::StartBadge => stack_str(buf, format_args!("[{}]", tl.start)),
        Item::EndBadge => stack_str(buf, format_args!("[{}]", tl.end)),
        _ => &tl.step_size,
    }
}

fn rich(item: Item, text: &str) -> egui::RichText {
    let rich = egui::RichText::new(text).small();
    if item == Item::DateInfo {
        rich.monospace().strong()
    } else {
        rich
    }
}

fn text_width(ui: &egui::Ui, item: Item, text: &str) -> f32 {
    egui::WidgetText::from(rich(item, text))
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Small,
        )
        .size()
        .x
}

/// Width reserved for the date value: the widest of the start, end and
/// current labels, so the bar layout stays still while the step changes.
fn date_value_width(ui: &egui::Ui, tl: &Timeline) -> f32 {
    [&tl.current, &tl.start, &tl.end]
        .into_iter()
        .map(|t| text_width(ui, Item::DateInfo, t))
        .fold(0.0, f32::max)
}

/// (full, compact) widths of a text item, without trailing spacing. The
/// date keeps its `Xs` hourglass icon when compact; whether an item hides
/// entirely is decided by the caller.
pub(super) fn widths(item: Item, tl: &Timeline, ui: &egui::Ui) -> (f32, f32) {
    if item == Item::DateInfo {
        let icon = IconSize::Xs.px();
        let spacing = ui.spacing().item_spacing.x;
        return (icon + spacing + date_value_width(ui, tl), icon);
    }
    let mut buf: LabelBuf = [0; LABEL_BUF_LEN];
    let full = text_width(ui, item, text(&mut buf, item, tl));
    (full, full)
}

/// Draw a text item. The date (icon, plus the current value unless compact)
/// carries the timeline details popover.
pub(super) fn show(item: Item, compact: bool, tl: &Timeline, app: &OctantApp, ui: &mut egui::Ui) {
    let mut buf: LabelBuf = [0; LABEL_BUF_LEN];
    let label = rich(item, text(&mut buf, item, tl));
    if item != Item::DateInfo {
        ui.label(label);
        return;
    }
    ui.horizontal(|ui| {
        ui.icon(Icon::Hourglass, IconSize::Xs);
        if !compact {
            // Pad to the reserved width so neighbours never shift.
            let used = ui.label(label).rect.width();
            ui.add_space((date_value_width(ui, tl) - used).max(0.0));
        }
    })
    .response
    .on_hover_ui(|ui| timeline_details(ui, tl, app));
}

/// Popover body; formatted only while it is shown.
fn timeline_details(ui: &mut egui::Ui, tl: &Timeline, app: &OctantApp) {
    ui.label(egui::RichText::new("Timeline Details").strong());
    ui.label(format!("Dimension: {}", dim_name(app)));
    ui.label(format!("Step: {} / {}", tl.step, tl.last_step));
    ui.label(format!("Current: {}", tl.current));
    ui.label(format!("Range: {} -> {}", tl.start, tl.end));
    ui.label(format!("Step Size: {}", tl.step_size));
}
