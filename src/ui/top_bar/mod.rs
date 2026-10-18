//! Top navigation bar. Items collapse to icon-only buttons as the window
//! narrows (see [`layout::COLLAPSE_ORDER`]) instead of being hidden.

mod items;
mod layout;
#[cfg(test)]
mod tests;

use super::status::{self, FetchProgress};
use crate::app::OctantApp;
use crate::ui::keyboard::regions::Region;
use crate::ui::toolbar::ItemWidths;
use crate::ui::toolbar::roving::{BarNav, Stop};
use layout::{ITEM_COUNT, LEFT_ITEMS, TopBarItem};

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
    TopBarItem::Language,
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

    let mut nav = BarNav::default();
    let show = |item, app: &mut OctantApp, ui: &mut egui::Ui, nav: &mut BarNav| {
        items::show_item(item, compact.get(item), app, ui, nav);
    };
    show(TopBarItem::Brand, app, ui, &mut nav);
    for item in LEFT_ITEMS {
        show(item, app, ui, &mut nav);
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        show(TopBarItem::Theme, app, ui, &mut nav);
        ui.separator();
        show(TopBarItem::Language, app, ui, &mut nav);
        ui.separator();
        show(TopBarItem::Cache, app, ui, &mut nav);

        if let Some(progress) = progress {
            ui.separator();
            let abort = status::show_status_bar(app, ui, progress, compact.get(TopBarItem::Status));
            nav.push(&abort, Stop::Button);
        }
    });
    nav.finish(ui, Region::TopBar);
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
