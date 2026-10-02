//! Top navigation bar. Items collapse to icon-only buttons as the window
//! narrows (see [`layout::COLLAPSE_ORDER`]) instead of being hidden.

mod items;
mod layout;
#[cfg(test)]
mod tests;

use super::status::{self, FetchProgress};
use crate::app::OctantApp;
use layout::{ITEM_COUNT, ItemWidths, LEFT_ITEMS, TopBarItem};

/// Minimum empty space kept between the left and right item groups.
const GROUP_GAP: f32 = 8.0;

const ALL_ITEMS: [TopBarItem; ITEM_COUNT] = [
    TopBarItem::Brand,
    TopBarItem::Dataset,
    TopBarItem::Variables,
    TopBarItem::Dimensions,
    TopBarItem::PlotType,
    TopBarItem::Colormap,
    TopBarItem::Settings,
    TopBarItem::Status,
    TopBarItem::Cache,
    TopBarItem::Theme,
];

pub fn show_top_bar(app: &mut OctantApp, ui: &mut egui::Ui) {
    egui::Panel::top("octant_top_bar")
        .exact_size(34.0)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| show_contents(app, ui));
        });
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
    let compact = layout::compute_compact(&widths, ui.available_width() - GROUP_GAP);

    items::show_item(TopBarItem::Brand, compact.get(TopBarItem::Brand), app, ui);
    for item in LEFT_ITEMS {
        items::show_item(item, compact.get(item), app, ui);
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        items::show_item(TopBarItem::Theme, compact.get(TopBarItem::Theme), app, ui);
        ui.separator();
        items::show_item(TopBarItem::Cache, compact.get(TopBarItem::Cache), app, ui);

        if let Some(progress) = progress {
            ui.separator();
            status::show_status_bar(app, ui, progress, compact.get(TopBarItem::Status));
        }
    });
}

fn status_widths(ui: &egui::Ui, progress: Option<FetchProgress>, spacing: f32) -> ItemWidths {
    let Some(progress) = progress else {
        return ItemWidths::default();
    };
    let trailing = spacing * 2.0 + items::SEPARATOR_WIDTH;
    ItemWidths {
        full: progress.width(ui, false) + trailing,
        compact: progress.width(ui, true) + trailing,
    }
}
