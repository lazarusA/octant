//! The plot type picker: a modal window in the middle of the screen, opened
//! by `P`, with a row per plot type. A number key picks the type it shows,
//! Up/Down move between the pickable rows and Enter picks the focused one;
//! types that cannot be picked show why. Picking closes it, as does Escape.

use super::options::{PlotOption, blocked_text, plot_options};
use super::plot_type_icon;
use crate::app::OctantApp;
use crate::i18n::{CommonKey, KeymapKey as K, tr};
use crate::plots::PlotType;
use crate::ui::icons::UiIconExt;
use crate::ui::keyboard::list_nav::ListNav;
use crate::ui::keyboard::{focus, windows};
use crate::utils::stack_str;
use egui::text::{LayoutJob, TextWrapping};
use egui::{Align2, Color32, FontId, Galley, Id, Key, Rect, Response, Sense, TextStyle, Ui, vec2};
use std::sync::Arc;

/// Width of the window's content.
pub(super) const WIDTH: f32 = 340.0;
const ROW_H: f32 = 40.0;
const PAD: f32 = 10.0;
/// The keys picking each row, in `options::ORDER`.
const DIGITS: [Key; 6] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
];

fn window_id() -> Id {
    Id::unique("plot_type_picker")
}

fn row_id(index: usize) -> Id {
    window_id().with(("row", index))
}

pub fn show_plot_type_picker(app: &mut OctantApp, ctx: &egui::Context) {
    let open = app.layout.show_plot_picker;
    let opened = windows::follow_window(ctx, window_id(), open);
    if !open {
        return;
    }
    let options = plot_options(app);
    let current = app.selected.plot_type;
    let (mut picked, mut close) = (None, false);
    egui::Window::new(tr(K::PickerTitle))
        .id(window_id())
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            windows::trap(ui, opened);
            ui.set_width(WIDTH);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr(K::PickerTitle)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = ui.close_button(tr(CommonKey::CloseEsc)).clicked();
                });
            });
            ui.label(egui::RichText::new(tr(K::PickerHint)).small().weak());
            ui.separator();
            picked = rows(ui, &options, current, opened);
        });
    if let Some(plot_type) = picked.or_else(|| digit_pick(ctx, &options)) {
        app.switch_plot_type(plot_type);
        close = true;
    }
    if close || ctx.input(|i| i.key_pressed(Key::Escape)) {
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

/// The rows; focus starts on the `current` type once the picker `opened`.
/// The type a row's click (or Enter) picked.
fn rows(
    ui: &mut Ui,
    options: &[PlotOption; 6],
    current: PlotType,
    opened: bool,
) -> Option<PlotType> {
    let start = options
        .iter()
        .position(|o| o.plot_type == current && o.blocked.is_none())
        .or_else(|| options.iter().position(|o| o.blocked.is_none()));
    if let Some(index) = start.filter(|_| opened) {
        focus::request(ui.ctx(), row_id(index));
    }
    let mut nav = ListNav::new(ui.ctx());
    let mut picked = None;
    for (index, option) in options.iter().enumerate() {
        let response = row(ui, index, *option, option.plot_type == current);
        if option.blocked.is_some() {
            continue;
        }
        nav.item(&response);
        if response.clicked() {
            picked = Some(option.plot_type);
        }
    }
    nav.finish(ui.ctx());
    picked
}

/// Row `index`: its number key, icon and name, and why it cannot be picked
/// beneath the name when it cannot; `selected` for the current type.
fn row(ui: &mut Ui, index: usize, option: PlotOption, selected: bool) -> Response {
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), ROW_H));
    let sense = match option.blocked {
        None => Sense::click(),
        Some(_) => Sense::hover(),
    };
    let response = ui.interact(rect, row_id(index), sense);
    let visuals = ui.visuals();
    let radius = visuals.widgets.hovered.corner_radius;
    if selected {
        ui.painter()
            .rect_filled(rect, radius, visuals.selection.bg_fill);
    } else if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, radius, visuals.widgets.hovered.weak_bg_fill);
    }
    focus::paint_ring(ui, &response, radius);
    paint_row_content(ui, rect, index, option);
    response
}

fn paint_row_content(ui: &Ui, rect: Rect, index: usize, option: PlotOption) {
    let visuals = ui.visuals();
    let (strong, weak) = (visuals.strong_text_color(), visuals.weak_text_color());
    let body = TextStyle::Body.resolve(ui.style());
    let painter = ui.painter();
    let mut digit = [0; 4];
    let digit = stack_str(&mut digit, format_args!("{}", index + 1));
    let digit_at = rect.left_center() + vec2(PAD, 0.0);
    painter.text(digit_at, Align2::LEFT_CENTER, digit, body.clone(), weak);
    let icon_at = rect.left_center() + vec2(PAD + 26.0, 0.0);
    let icon = Rect::from_center_size(icon_at, vec2(18.0, 18.0));
    let tone = if option.blocked.is_some() {
        weak
    } else {
        strong
    };
    plot_type_icon(option.plot_type).paint(painter, icon, tone, visuals.dark_mode);
    let text_x = icon.right() + PAD;
    let width = rect.right() - PAD - text_x;
    let name = option.plot_type.display_name();
    let Some(blocked) = option.blocked else {
        let galley = line(ui, name, body, width);
        painter.galley(
            egui::pos2(text_x, rect.center().y - galley.size().y / 2.0),
            galley,
            strong,
        );
        return;
    };
    let name = line(ui, name, body, width);
    let mut buf = [0; 256];
    let reason = line(
        ui,
        blocked_text(&mut buf, blocked),
        TextStyle::Small.resolve(ui.style()),
        width,
    );
    let top = rect.center().y - (name.size().y + reason.size().y) / 2.0;
    let below = top + name.size().y;
    painter.galley(egui::pos2(text_x, top), name, weak);
    painter.galley(egui::pos2(text_x, below), reason, weak);
}

/// `text` on one line in `font`, cut with an ellipsis past `width`.
fn line(ui: &Ui, text: &str, font: FontId, width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, Color32::PLACEHOLDER);
    job.wrap = TextWrapping::truncate_at_width(width);
    ui.painter().layout_job(job)
}
