//! The plot type picker, opened by `P`: a semi-transparent strip in the
//! middle of the screen, the plot types as icon tiles in a row under the
//! name of the one hovered (else focused, else current). A number key picks
//! the type of its tile, the arrows move between the pickable tiles and
//! Enter picks the focused one; a type that cannot be picked shows why under
//! its name. Picking closes it, as do Escape and a click outside.

use super::options::{PlotOption, blocked_text, plot_options};
use super::plot_type_icon;
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::icons::IconSize;
use crate::ui::keyboard::{focus, windows};
use crate::utils::stack_str;
use egui::text::{LayoutJob, TextWrapping};
use egui::{Align2, Color32, FontId, Galley, Id, Key, Rect, Response, Sense, TextStyle, Ui, vec2};
use std::sync::Arc;

/// Side of a tile.
const TILE: f32 = 52.0;
/// Gap between tiles.
const GAP: f32 = 6.0;
/// Width of the row of tiles, the picker's content width.
pub(super) const WIDTH: f32 = TILE * 6.0 + GAP * 5.0;
/// Opacity of the picker's fill, so the plot shows through.
const FILL_ALPHA: f32 = 0.8;
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
                picked = contents(ui, &options, current, opened);
            });
        });
    let outside = ctx.input(|i| {
        i.pointer.primary_clicked()
            && i.pointer
                .interact_pos()
                .is_some_and(|p| !area.response.rect.contains(p))
    });
    let mut close = outside || ctx.input(|i| i.key_pressed(Key::Escape));
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

/// The name of the type shown above its row of tiles; the type a tile's
/// click (or Enter) picked.
fn contents(
    ui: &mut Ui,
    options: &[PlotOption; 6],
    current: PlotType,
    opened: bool,
) -> Option<PlotType> {
    let body = TextStyle::Body.resolve(ui.style());
    let small = TextStyle::Small.resolve(ui.style());
    let label_h = ui.fonts_mut(|f| f.row_height(&body) + f.row_height(&small));
    let (_, label) = ui.allocate_space(vec2(WIDTH, label_h));
    let current_index = options.iter().position(|o| o.plot_type == current);
    let start = current_index
        .filter(|&i| options[i].blocked.is_none())
        .or_else(|| options.iter().position(|o| o.blocked.is_none()));
    if let Some(index) = start.filter(|_| opened) {
        focus::request(ui.ctx(), tile_id(index));
    }
    let row = tiles(ui, options, current);
    if let Some(shown) = row.hovered.or(row.focused).or(current_index) {
        paint_label(ui, label, options[shown], (body, small));
    }
    row.picked
}

/// What the row of tiles reported this frame.
struct Row {
    hovered: Option<usize>,
    focused: Option<usize>,
    picked: Option<PlotType>,
}

/// The tiles, and the arrows moving focus between the pickable ones.
fn tiles(ui: &mut Ui, options: &[PlotOption; 6], current: PlotType) -> Row {
    let mut row = Row {
        hovered: None,
        focused: None,
        picked: None,
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        for (index, option) in options.iter().enumerate() {
            let response = tile(ui, index, *option, option.plot_type == current);
            row.hovered = row.hovered.or(response.hovered().then_some(index));
            row.focused = row.focused.or(response.has_focus().then_some(index));
            if response.clicked() {
                row.picked = Some(option.plot_type);
            }
        }
    });
    if let Some(at) = row.focused {
        focus::claim_arrows(ui.ctx(), tile_id(at));
        let step = focus::pressed_arrow(ui.ctx()).map(|d| d.x + d.y);
        let pickable = |&i: &usize| options[i].blocked.is_none();
        let next = match step {
            Some(s) if s > 0.0 => (at + 1..options.len()).find(pickable),
            Some(s) if s < 0.0 => (0..at).rev().find(pickable),
            _ => None,
        };
        if let Some(next) = next {
            focus::request(ui.ctx(), tile_id(next));
        }
    }
    row
}

/// Tile `index`: its type's icon and number key, `selected` for the current
/// type; only pickable types take clicks and focus.
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
    focus::paint_ring(ui, &response, radius);
    response
}

/// `option`'s name centered at the top of `rect`, and why it cannot be
/// picked beneath it when it cannot.
fn paint_label(ui: &Ui, rect: Rect, option: PlotOption, (body, small): (FontId, FontId)) {
    let visuals = ui.visuals();
    let painter = ui.painter();
    let centered =
        |galley: &Arc<Galley>, y: f32| egui::pos2(rect.center().x - galley.size().x / 2.0, y);
    let name = line(ui, option.plot_type.display_name(), body, rect.width());
    let Some(blocked) = option.blocked else {
        painter.galley(
            centered(&name, rect.top()),
            name,
            visuals.strong_text_color(),
        );
        return;
    };
    let mut buf = [0; 256];
    let reason = line(ui, blocked_text(&mut buf, blocked), small, rect.width());
    let weak = visuals.weak_text_color();
    let below = rect.top() + name.size().y;
    painter.galley(centered(&name, rect.top()), name, weak);
    painter.galley(centered(&reason, below), reason, weak);
}

/// `text` on one line in `font`, cut with an ellipsis past `width`.
fn line(ui: &Ui, text: &str, font: FontId, width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, Color32::PLACEHOLDER);
    job.wrap = TextWrapping::truncate_at_width(width);
    ui.painter().layout_job(job)
}
