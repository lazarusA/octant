//! Top navigation bar. Items collapse to icon-only buttons as the window
//! narrows (see [`layout::COLLAPSE_ORDER`]); when even the icons do not fit,
//! the right-hand items move into the More menu (`more.rs`). Dragging its
//! lower edge up hides it to a thin strip (`hide.rs`), as the bottom bar.

mod hide;
mod items;
mod layout;
mod more;
#[cfg(test)]
mod tests;

use super::status::{self, FetchProgress};
use crate::app::OctantApp;
use crate::ui::keyboard::regions::Region;
use crate::ui::toolbar::ItemWidths;
use crate::ui::toolbar::roving::{BarNav, Stop};
use layout::{CompactFlags, ITEM_COUNT, LEFT_ITEMS, TopBarItem};

/// Height of the shown bar.
const BAR_HEIGHT: f32 = 34.0;

/// Minimum empty space kept between the left and right item groups.
const GROUP_GAP: f32 = 8.0;

const ALL_ITEMS: [TopBarItem; ITEM_COUNT] = [
    TopBarItem::Brand,
    TopBarItem::Dataset,
    TopBarItem::Variables,
    TopBarItem::Dimensions,
    TopBarItem::PlotType,
    TopBarItem::Settings,
    TopBarItem::Status,
    TopBarItem::Cache,
    TopBarItem::Voice,
    TopBarItem::Language,
    TopBarItem::Theme,
];

pub fn show_top_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    // Local copy: the panel closure borrows all of `app`.
    let mut expanded = app.layout.show_top_bar;
    let mut request_expand = false;
    egui::Panel::show_switched(
        ui,
        &mut expanded,
        hide::strip_panel(ui),
        egui::Panel::top("octant_top_bar")
            .resizable(true)
            .default_size(BAR_HEIGHT)
            // Its size the top of the range, so the edge's cursor points up:
            // the bar only shrinks, to hide (an exact size would point down).
            .size_range(BAR_HEIGHT - 0.5..=BAR_HEIGHT),
        |ui, is_expanded| {
            if is_expanded {
                ui.horizontal_centered(|ui| show_contents(app, ui));
            } else {
                request_expand = hide::show_strip(ui);
            }
        },
    );
    app.layout.show_top_bar = expanded || request_expand;
}

fn show_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    let spacing = ui.spacing().item_spacing.x;
    let progress = FetchProgress::from_app(app);

    let mut widths = [ItemWidths::default(); ITEM_COUNT];
    for item in ALL_ITEMS {
        widths[item as usize] = match item {
            TopBarItem::Status => status_widths(ui, progress, spacing),
            _ => items::item_widths(item, app, ui),
        };
    }
    let available = ui.available_width() - GROUP_GAP;
    let overflow = layout::overflows(&widths, available);
    let compact = if overflow {
        CompactFlags::all()
    } else {
        layout::compute_compact(&widths, available)
    };

    let mut nav = BarNav::default();
    let show = |item, app: &mut OctantApp, ui: &mut egui::Ui, nav: &mut BarNav| {
        items::show_item(item, compact.get(item), app, ui, nav);
    };
    show(TopBarItem::Brand, app, ui, &mut nav);
    for item in LEFT_ITEMS {
        show(item, app, ui, &mut nav);
    }

    let layout = egui::Layout::right_to_left(egui::Align::Center);
    ui.with_layout(layout, |ui| {
        show_right_group(app, ui, &mut nav, compact, overflow, progress);
    });
    // Not while sliding away hidden, so focus asked for the top bar goes to
    // the strip.
    if app.layout.show_top_bar {
        nav.finish(ui, Region::TopBar);
    }
}

/// The right-aligned items, right to left: the items or the More menu
/// holding them, then the fetch status.
fn show_right_group(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    nav: &mut BarNav,
    compact: CompactFlags,
    overflow: bool,
    progress: Option<FetchProgress>,
) {
    if overflow {
        more::show_more_menu(app, ui, nav);
    } else {
        let items = [
            TopBarItem::Theme,
            TopBarItem::Language,
            TopBarItem::Voice,
            TopBarItem::Cache,
        ];
        for (i, item) in items.into_iter().enumerate() {
            if i > 0 {
                ui.separator();
            }
            items::show_item(item, compact.get(item), app, ui, nav);
        }
    }
    if let Some(progress) = progress {
        ui.separator();
        let abort = status::show_status_bar(app, ui, progress, compact.get(TopBarItem::Status));
        nav.push(&abort, Stop::Button);
    }
}

fn status_widths(ui: &egui::Ui, progress: Option<FetchProgress>, spacing: f32) -> ItemWidths {
    let Some(progress) = progress else {
        return ItemWidths::default();
    };
    let trailing = spacing * 2.0 + crate::ui::toolbar::SEPARATOR_WIDTH;
    ItemWidths {
        full: progress.width(ui, false) + trailing,
        compact: progress.width(ui, true) + trailing,
    }
}
