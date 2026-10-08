//! Draws the toast stack in the bottom-right corner of the canvas.

use egui::text::LayoutJob;
use egui::{Align2, Frame, Id, Margin, Order, Rect, RichText, Stroke, Ui};
use web_time::Instant;

use super::model::{Severity, Toast, ToastAction};
use super::queue::{MAX_TOASTS, Toasts, opacity};
use crate::ui::icons::{Icon, IconSize, ToolbarButton, UiIconExt};

/// Width of a toast, in points.
const WIDTH: f32 = 340.0;
/// Gap between toasts and to the canvas edges.
const GAP: f32 = 10.0;

/// Draws the toasts over `canvas` and runs their clock to `now`; returns the
/// action the user clicked, if any. Schedules the repaints the stack needs.
pub fn show(
    ctx: &egui::Context,
    toasts: &mut Toasts,
    canvas: Rect,
    now: Instant,
) -> Option<ToastAction> {
    if toasts.is_empty() {
        return None;
    }
    let mut hovered = [false; MAX_TOASTS];
    let mut dismissed = None;
    let mut clicked = None;
    let width = WIDTH.min(canvas.width() - 2.0 * GAP).max(120.0);
    egui::Area::new(Id::new("octant_toasts"))
        .order(Order::Foreground)
        .fade_in(false)
        .pivot(Align2::RIGHT_BOTTOM)
        .fixed_pos(canvas.right_bottom() - egui::vec2(GAP, GAP))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = GAP * 0.6;
            for (i, toast) in toasts.items().iter().enumerate() {
                let card = ui.scope(|ui| {
                    ui.multiply_opacity(opacity(toast.remaining));
                    toast_card(ui, toast, width, &mut dismissed, &mut clicked)
                });
                if let Some(flag) = hovered.get_mut(i) {
                    *flag = ui.rect_contains_pointer(card.response.rect);
                }
            }
        });
    if let Some(id) = dismissed {
        toasts.dismiss(id);
    }
    toasts.tick(now, &hovered);
    match toasts.next_repaint() {
        Some(wait) if wait.is_zero() => ctx.request_repaint(),
        Some(wait) => ctx.request_repaint_after(wait),
        None => {}
    }
    clicked
}

/// One toast: icon, title (with its repeat count), detail and buttons.
fn toast_card(
    ui: &mut Ui,
    toast: &Toast,
    width: f32,
    dismissed: &mut Option<u64>,
    clicked: &mut Option<ToastAction>,
) {
    let notice = &toast.notice;
    let tone = notice.severity.tone().color(ui.visuals());
    Frame::popup(ui.style())
        .stroke(Stroke::new(1.0, tone))
        .inner_margin(Margin::symmetric(10, 8))
        .corner_radius(6.0)
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.horizontal_top(|ui| {
                ui.icon_toned(notice.severity.icon(), IconSize::Sm, notice.severity.tone());
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 2.0 * IconSize::Sm.px() - 12.0);
                    title_row(ui, toast);
                    if !notice.detail.is_empty() {
                        detail_label(ui, &notice.detail);
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.close_button("Dismiss").clicked() {
                        *dismissed = Some(toast.id);
                    }
                    if notice.severity == Severity::Error
                        && !notice.detail.is_empty()
                        && copy_button(ui).clicked()
                    {
                        ui.ctx()
                            .copy_text(format!("{}: {}", notice.title, notice.detail));
                    }
                    if let Some(action) = &notice.action
                        && action_button(ui, action).clicked()
                    {
                        *clicked = Some(action.clone());
                        *dismissed = Some(toast.id);
                    }
                });
            });
        });
}

fn title_row(ui: &mut Ui, toast: &Toast) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(RichText::new(&toast.notice.title).strong());
        if toast.count > 1 {
            ui.label(RichText::new(format!("x{}", toast.count)).small().weak());
        }
    });
}

/// The detail in at most two lines; the full text shows on hover.
fn detail_label(ui: &mut Ui, detail: &str) {
    let color = ui.visuals().weak_text_color();
    let font = egui::TextStyle::Small.resolve(ui.style());
    let mut job = LayoutJob::simple(detail.to_owned(), font, color, ui.available_width());
    job.wrap.max_rows = 2;
    ui.label(job).on_hover_text(detail);
}

fn copy_button(ui: &mut Ui) -> egui::Response {
    ui.add(
        ToolbarButton::new(Icon::Clipboard, "Copy details")
            .compact(true)
            .icon_size(IconSize::Sm),
    )
}

fn action_button(ui: &mut Ui, action: &ToastAction) -> egui::Response {
    match action {
        ToastAction::RevealFile(path) => {
            #[cfg(target_os = "macos")]
            let label = "Reveal in Finder";
            #[cfg(not(target_os = "macos"))]
            let label = "Open Folder";
            let hover = format!("{label}: {}", path.display());
            ui.add(
                ToolbarButton::new(Icon::FolderOpen, label)
                    .compact(true)
                    .hover(&hover)
                    .icon_size(IconSize::Sm),
            )
        }
    }
}
