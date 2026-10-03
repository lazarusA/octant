//! Text items of the bottom bar: the current-step date (hourglass icon plus
//! value, with a details popover) and the start, step-size and end badges.

use super::layout::BottomBarItem as Item;
use super::timeline::{Timeline, dim_name};
use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, UiIconExt};
use crate::ui::toolbar::ItemWidths;
use crate::utils::stack_str;

/// Stack buffer for date and badge labels.
type LabelBuf = [u8; 160];

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

/// Widths of a text item, including trailing spacing. Badges hide when
/// compact (zero width); the date keeps just its icon.
pub(super) fn widths(item: Item, tl: &Timeline, ui: &egui::Ui) -> ItemWidths {
    let spacing = ui.spacing().item_spacing.x;
    // The date line leads with an `Xs` hourglass icon.
    let icon = if item == Item::DateInfo {
        IconSize::Xs.px() + spacing
    } else {
        0.0
    };
    let mut buf = [0u8; 160];
    let full = icon + text_width(ui, item, text(&mut buf, item, tl)) + spacing;
    let compact = if item == Item::DateInfo { icon } else { 0.0 };
    ItemWidths { full, compact }
}

/// Draw a text item. The date (icon, plus the current value unless compact)
/// carries the timeline details popover.
pub(super) fn show(item: Item, compact: bool, tl: &Timeline, app: &OctantApp, ui: &mut egui::Ui) {
    let mut buf = [0u8; 160];
    let label = rich(item, text(&mut buf, item, tl));
    if item != Item::DateInfo {
        ui.label(label);
        return;
    }
    ui.horizontal(|ui| {
        ui.icon(Icon::Hourglass, IconSize::Xs);
        if !compact {
            ui.label(label);
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
