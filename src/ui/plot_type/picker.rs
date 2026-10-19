//! The plot type picker, opened by `P`: a semi-transparent strip in the
//! middle of the screen, the plot types as icon tiles in a row, each named
//! (with why it cannot be picked, when it cannot) by its tooltip on hover
//! and under keyboard focus. A number key picks the type of its tile, Left,
//! Right, Home and End move between the pickable tiles and Enter picks the
//! focused one.
//! Picking closes it, as do Escape and a click outside.

use super::options::{PlotOption, blocked_text, plot_options};
use super::plot_type_icon;
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::icons::IconSize;
use crate::ui::keyboard::{focus, windows};
use crate::utils::stack_str;
use egui::{Align2, Id, Key, Rect, Response, Sense, TextStyle, Ui, vec2};

/// Side of a tile.
const TILE: f32 = 52.0;
/// Gap between tiles.
const GAP: f32 = 6.0;
/// Width of the row of tiles, the picker's content width.
pub(super) const WIDTH: f32 = TILE * 6.0 + GAP * 5.0;
/// Opacity of the picker's fill, so the plot shows through.
const FILL_ALPHA: f32 = 0.8;
/// The keys moving focus along the row.
const ROW_KEYS: [Key; 4] = [Key::ArrowLeft, Key::ArrowRight, Key::Home, Key::End];
/// The keys picking each tile, in `options::ORDER`.
const DIGITS: [Key; 6] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
];

fn picker_id() -> Id {
    Id::unique("plot_type_picker")
}

fn tile_id(index: usize) -> Id {
    picker_id().with(("tile", index))
}

pub fn show_plot_type_picker(app: &mut OctantApp, ctx: &egui::Context) {
    let open = app.layout.show_plot_picker;
    let opened = windows::follow_window(ctx, picker_id(), open);
    if !open {
        return;
    }
    let options = plot_options(app);
    let current = app.selected.plot_type;
    let mut picked = None;
    let area = egui::Area::new(picker_id())
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        // Summoned by a key: shown at once rather than faded in.
        .fade_in(false)
        .show(ctx, |ui| {
            windows::trap(ui, opened);
            let fill = ui.visuals().window_fill.gamma_multiply(FILL_ALPHA);
            egui::Frame::window(ui.style()).fill(fill).show(ui, |ui| {
                ui.set_width(WIDTH);
                // Focus starts on the current type, else the first pickable.
                let pickable = |o: &PlotOption| o.blocked.is_none();
                let start = options
                    .iter()
                    .position(|o| o.plot_type == current && pickable(o))
                    .or_else(|| options.iter().position(pickable));
                if let Some(index) = start.filter(|_| opened) {
                    focus::request(ui.ctx(), tile_id(index));
                }
                picked = tiles(ui, &options, current);
            });
        });
    let mut close = windows::dismissed(ctx, area.response.rect);
    if let Some(plot_type) = picked.or_else(|| digit_pick(ctx, &options)) {
        app.switch_plot_type(plot_type);
        close = true;
    }
    if close {
        app.layout.show_plot_picker = false;
    }
}

/// The plot type whose number key was pressed, when it can be picked.
fn digit_pick(ctx: &egui::Context, options: &[PlotOption; 6]) -> Option<PlotType> {
    let key = focus::pressed(ctx, DIGITS)?;
    let index = DIGITS.iter().position(|&k| k == key)?;
    options
        .get(index)
        .filter(|o| o.blocked.is_none())
        .map(|o| o.plot_type)
}

/// The tiles, `ROW_KEYS` moving focus between the pickable ones; the type
/// a tile's click (or Enter) picked.
fn tiles(ui: &mut Ui, options: &[PlotOption; 6], current: PlotType) -> Option<PlotType> {
    let (mut picked, mut pressed) = (None, None);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for (index, option) in options.iter().enumerate() {
            let response = tile(ui, index, *option, option.plot_type == current);
            if response.has_focus() {
                focus::claim_arrows_only(ui.ctx(), response.id);
                pressed = focus::pressed(ui.ctx(), ROW_KEYS).map(|key| (index, key));
            }
            if response.clicked() {
                picked = Some(option.plot_type);
            }
        }
    });
    // Blocked tiles take no focus, so the moves skip them.
    let pickable = |&i: &usize| options[i].blocked.is_none();
    let len = options.len();
    let next = pressed.and_then(|(at, key)| match key {
        Key::ArrowLeft => (0..at).rev().find(pickable),
        Key::ArrowRight => (at + 1..len).find(pickable),
        Key::Home => (0..len).find(pickable),
        Key::End => (0..len).rev().find(pickable),
        _ => None,
    });
    if let Some(next) = next {
        focus::request(ui.ctx(), tile_id(next));
    }
    picked
}

/// Tile `index`: its type's icon and number key, `selected` for the current
/// type, its name (and why it cannot be picked) as its tooltip, shown on
/// hover and under keyboard focus; only pickable types take clicks and
/// focus.
fn tile(ui: &mut Ui, index: usize, option: PlotOption, selected: bool) -> Response {
    let (_, rect) = ui.allocate_space(vec2(TILE, TILE));
    let pickable = option.blocked.is_none();
    let sense = if pickable {
        Sense::click()
    } else {
        Sense::hover()
    };
    let response = ui.interact(rect, tile_id(index), sense);
    let name = option.plot_type.display_name();
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, pickable, selected, name));
    let visuals = ui.visuals();
    let radius = visuals.widgets.hovered.corner_radius;
    if selected {
        ui.painter()
            .rect_filled(rect, radius, visuals.selection.bg_fill);
    } else if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, radius, visuals.widgets.hovered.weak_bg_fill);
    }
    let (strong, weak) = (visuals.strong_text_color(), visuals.weak_text_color());
    let icon = Rect::from_center_size(rect.center(), egui::Vec2::splat(IconSize::Lg.px()));
    let tone = if pickable { strong } else { weak };
    plot_type_icon(option.plot_type).paint(ui.painter(), icon, tone, visuals.dark_mode);
    let mut digit = [0; 4];
    let digit = stack_str(&mut digit, format_args!("{}", index + 1));
    let corner = rect.right_bottom() - vec2(5.0, 3.0);
    let small = TextStyle::Small.resolve(ui.style());
    ui.painter()
        .text(corner, Align2::RIGHT_BOTTOM, digit, small, weak);
    focus::ring_and_tooltip(ui, response, radius, |ui| tooltip(ui, option))
}

/// A tile's tooltip: the type's name, and why it cannot be picked.
fn tooltip(ui: &mut Ui, option: PlotOption) {
    ui.label(egui::RichText::new(option.plot_type.display_name()).strong());
    if let Some(blocked) = option.blocked {
        let mut buf = [0; 256];
        ui.label(
            egui::RichText::new(blocked_text(&mut buf, blocked))
                .small()
                .weak(),
        );
    }
}
