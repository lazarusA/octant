//! A colorbar's Color window, opened by the cog in its top-left corner:
//! the layer's Color menu (label, range, scale, NaN and clip colors,
//! opacity and alpha curve) and its Show Colorbar checkbox beside the
//! panel, until the cog, its close button (top-right) or Escape closes it. A
//! floating area rather than an egui popup, so the scale select and color
//! pickers inside it do not close it.

use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use crate::i18n::{ColormapKey, SettingsKey as K, tr};
use crate::ui::icons::{IconSize, ToolbarButton, UiIconExt};
use crate::ui::keyboard::{focus, windows};
use crate::ui::panel_layout::{GAP, PANEL_W};
use egui::{Align2, Rect, Vec2};

/// Shortest the scrolled body gets on a very short canvas.
const MIN_BODY_HEIGHT: f32 = 120.0;

fn open_key(id: LayerId) -> egui::Id {
    egui::Id::unique(super::salt("colorbar_color_menu_open", id))
}

/// Egui id of layer `id`'s Color window area.
pub(super) fn area_id(id: LayerId) -> egui::Id {
    egui::Id::unique(super::salt("colorbar_color_menu", id))
}

/// Whether layer `id`'s Color window is open.
pub(super) fn is_open(ctx: &egui::Context, id: LayerId) -> bool {
    ctx.data(|d| d.get_temp(open_key(id)).unwrap_or(false))
}

pub(super) fn set_open(ctx: &egui::Context, id: LayerId, open: bool) {
    ctx.data_mut(|d| d.insert_temp(open_key(id), open));
}

/// Layer `id`'s Color window, when open, beside its colorbar `panel` on
/// `canvas`: above or below a horizontal bar, left or right of a vertical
/// one, on the side with more room. Never during export.
pub(super) fn show(
    app: &mut OctantApp,
    ctx: &egui::Context,
    id: LayerId,
    (panel, orientation): (Rect, BarOrientation),
    canvas: Rect,
) {
    let open = is_open(ctx, id);
    // Focus returns to the cog once the window closes.
    windows::follow_window(ctx, area_id(id), open);
    if app.pending_export.is_some() || !open {
        return;
    }
    let (pivot, point, room) = placement(panel, orientation, canvas);
    let mut close = false;
    egui::Area::new(area_id(id))
        .order(egui::Order::Foreground)
        .pivot(pivot)
        .fixed_pos(point)
        .constrain_to(canvas)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(PANEL_W);
                let top = ui.cursor().min.y;
                let max_height = (room - 2.0 * GAP).max(MIN_BODY_HEIGHT);
                // The area lends its last-frame size; open up to the room
                // left so the body can grow when a section expands.
                ui.set_max_height(max_height);
                egui::ScrollArea::vertical()
                    .id_salt(("colorbar_color_menu_scroll", id))
                    .max_height(max_height)
                    .show(ui, |ui| {
                        crate::ui::settings::show_color_menu(app, ui, id);
                        ui.separator();
                        show_colorbar_checkbox(app, ui, id);
                    });
                close = close_button(ui, top);
            });
        });
    // Hiding the colorbar closes its window too, as does Escape in it.
    let escaped = focus::escaped_layer(ctx).is_some_and(|l| l.id == area_id(id));
    if close || escaped || app.layers.get(id).is_some_and(|l| !l.colorbar.shown) {
        set_open(ctx, id, false);
    }
}

/// Layer `id`'s Show Colorbar checkbox: unticked, the colorbar and this
/// window close, until the layer's entry in Settings' Layers menu shows it
/// again.
fn show_colorbar_checkbox(app: &mut OctantApp, ui: &mut egui::Ui, id: LayerId) {
    if let Some(layer) = app.layers.get_mut(id) {
        ui.checkbox(&mut layer.colorbar.shown, tr(ColormapKey::ShowColorbar));
    }
}

/// The window's pivot, its point and the room it has along the bar's
/// cross axis.
fn placement(panel: Rect, orientation: BarOrientation, canvas: Rect) -> (Align2, egui::Pos2, f32) {
    let gap = Vec2::splat(GAP);
    match orientation {
        BarOrientation::Horizontal if panel.center().y > canvas.center().y => (
            Align2::LEFT_BOTTOM,
            panel.left_top() - Vec2::new(0.0, gap.y),
            panel.min.y - canvas.min.y,
        ),
        BarOrientation::Horizontal => (
            Align2::LEFT_TOP,
            panel.left_bottom() + Vec2::new(0.0, gap.y),
            canvas.max.y - panel.max.y,
        ),
        BarOrientation::Vertical if panel.center().x > canvas.center().x => (
            Align2::RIGHT_TOP,
            panel.left_top() - Vec2::new(gap.x, 0.0),
            canvas.max.y - panel.min.y,
        ),
        BarOrientation::Vertical => (
            Align2::LEFT_TOP,
            panel.right_top() + Vec2::new(gap.x, 0.0),
            canvas.max.y - panel.min.y,
        ),
    }
}

/// The close button in the window's top-right corner, beside the label
/// field of the first row (no title row of its own); whether it was clicked.
fn close_button(ui: &mut egui::Ui, top: f32) -> bool {
    let side = ToolbarButton::side(IconSize::Sm);
    let corner = egui::pos2(ui.max_rect().right() - side, top);
    let rect = Rect::from_min_size(corner, Vec2::splat(side));
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.close_button(tr(K::CloseColorHover)).clicked()
    })
    .inner
}
