//! The color picker's saturation/value plane and its hue and alpha strips:
//! dragged or clicked with the pointer, and moved with the arrow keys while
//! focused (the plane in both directions, the strips left and right).

use crate::ui::keyboard::{focus, keymap};
use egui::{Color32, Key, Pos2, Rect, Response, Shape, Stroke, Ui, Vec2, ecolor::Hsva};

/// How far one arrow press moves a value along its 0..1 range.
const KEY_STEP: f32 = 0.01;

/// The arrow pressed on focused `response`, as a direction (y down).
fn arrow(response: &Response) -> Option<Vec2> {
    if !response.has_focus() {
        return None;
    }
    focus::claim_arrows(&response.ctx, response.id);
    let arrows = [
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    ];
    keymap::arrow_direction(focus::pressed(&response.ctx, arrows)?)
}

/// Where the pointer drags or clicks `response`, as fractions of its rect.
fn pointer_at(ui: &Ui, response: &Response) -> Option<Vec2> {
    if !(response.dragged() || response.clicked()) {
        return None;
    }
    let pos = ui.input(|i| i.pointer.interact_pos())?;
    let rect = response.rect;
    Some(((pos - rect.min) / rect.size()).clamp(Vec2::ZERO, Vec2::splat(1.0)))
}

fn outline(ui: &Ui, rect: Rect) {
    let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, color), egui::StrokeKind::Inside);
}

/// A four-corner gradient over `rect`, colors from the top-left clockwise.
fn gradient(ui: &Ui, rect: Rect, [tl, tr, br, bl]: [Color32; 4]) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), tl);
    mesh.colored_vertex(rect.right_top(), tr);
    mesh.colored_vertex(rect.right_bottom(), br);
    mesh.colored_vertex(rect.left_bottom(), bl);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(Shape::mesh(mesh));
}

/// A strip's cursor at fraction `t` along `rect`.
fn strip_cursor(ui: &Ui, rect: Rect, t: f32) {
    let x = rect.min.x + t * rect.width();
    let cursor = Rect::from_center_size(
        Pos2::new(x, rect.center().y),
        Vec2::new(5.0, rect.height() + 4.0),
    );
    ui.painter().rect(
        cursor,
        2.0,
        Color32::WHITE,
        Stroke::new(1.0, Color32::BLACK),
        egui::StrokeKind::Middle,
    );
}

/// The saturation (across) and value (up) plane, `width` wide; whether
/// `hsva` changed.
pub(super) fn sv_plane(ui: &mut Ui, hsva: &mut Hsva, width: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 165.0), egui::Sense::drag());
    let mut changed = false;
    if let Some(at) = pointer_at(ui, &response) {
        hsva.s = at.x;
        hsva.v = 1.0 - at.y;
        changed = true;
    }
    if let Some(d) = arrow(&response) {
        hsva.s = (hsva.s + d.x * KEY_STEP).clamp(0.0, 1.0);
        hsva.v = (hsva.v - d.y * KEY_STEP).clamp(0.0, 1.0);
        changed = true;
    }

    let hue = Color32::from(Hsva::new(hsva.h, 1.0, 1.0, 1.0));
    gradient(
        ui,
        rect,
        [Color32::WHITE, hue, Color32::BLACK, Color32::BLACK],
    );
    outline(ui, rect);
    let cursor = Pos2::new(
        rect.min.x + hsva.s * rect.width(),
        rect.min.y + (1.0 - hsva.v) * rect.height(),
    );
    let painter = ui.painter();
    painter.circle(
        cursor,
        5.0,
        Color32::TRANSPARENT,
        Stroke::new(2.0, Color32::WHITE),
    );
    painter.circle(
        cursor,
        6.0,
        Color32::TRANSPARENT,
        Stroke::new(1.0, Color32::BLACK),
    );
    focus::paint_ring(ui, &response, 2.0);
    changed
}

/// A strip `width` wide for the 0..1 `value`; whether it changed, and the
/// strip's response.
fn strip(ui: &mut Ui, value: &mut f32, width: f32) -> (bool, Response) {
    let (_, response) = ui.allocate_exact_size(Vec2::new(width, 15.0), egui::Sense::drag());
    let mut changed = false;
    if let Some(at) = pointer_at(ui, &response) {
        *value = at.x;
        changed = true;
    }
    if let Some(d) = arrow(&response) {
        *value = (*value + d.x * KEY_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    (changed, response)
}

/// The hue spectrum strip; whether `hsva`'s hue changed.
pub(super) fn hue_strip(ui: &mut Ui, hsva: &mut Hsva, width: f32) -> bool {
    let (changed, response) = strip(ui, &mut hsva.h, width);
    let rect = response.rect;
    let segments = 6;
    for i in 0..segments {
        let (h0, h1) = (i as f32 / segments as f32, (i + 1) as f32 / segments as f32);
        let x = |h: f32| rect.min.x + h * rect.width();
        let part = Rect::from_x_y_ranges(x(h0)..=x(h1), rect.y_range());
        let c0 = Color32::from(Hsva::new(h0, 1.0, 1.0, 1.0));
        let c1 = Color32::from(Hsva::new(h1, 1.0, 1.0, 1.0));
        gradient(ui, part, [c0, c1, c1, c0]);
    }
    outline(ui, rect);
    strip_cursor(ui, rect, hsva.h);
    focus::paint_ring(ui, &response, 2.0);
    changed
}

/// The alpha strip over a checkerboard; whether `hsva`'s alpha changed.
pub(super) fn alpha_strip(ui: &mut Ui, hsva: &mut Hsva, width: f32) -> bool {
    let (changed, response) = strip(ui, &mut hsva.a, width);
    let rect = response.rect;
    let grid = 5.0;
    let mut x = rect.min.x;
    while x < rect.max.x {
        let col = ((x - rect.min.x) / grid) as usize;
        let mut y = rect.min.y;
        while y < rect.max.y {
            let row = ((y - rect.min.y) / grid) as usize;
            let gray = if (row + col).is_multiple_of(2) {
                180
            } else {
                240
            };
            let cell = Rect::from_min_max(
                Pos2::new(x, y),
                Pos2::new((x + grid).min(rect.max.x), (y + grid).min(rect.max.y)),
            );
            ui.painter()
                .rect_filled(cell, 0.0, Color32::from_gray(gray));
            y += grid;
        }
        x += grid;
    }
    let clear = Color32::from(Hsva::new(hsva.h, hsva.s, hsva.v, 0.0));
    let opaque = Color32::from(Hsva::new(hsva.h, hsva.s, hsva.v, 1.0));
    gradient(ui, rect, [clear, opaque, opaque, clear]);
    outline(ui, rect);
    strip_cursor(ui, rect, hsva.a);
    focus::paint_ring(ui, &response, 2.0);
    changed
}
