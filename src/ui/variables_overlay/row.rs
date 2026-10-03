//! Full-width clickable tree row shared by folders and variables:
//! `[chevron] [icon] name  detail`, where every pixel of the row is a hit target.

use crate::ui::icons::{Icon, IconSize, IconTone};
use crate::ui::key_focus;
use egui::{
    Color32, Id, Rect, Response, Sense, TextStyle, TextWrapMode, Ui, WidgetText, pos2, vec2,
};

/// Horizontal padding inside the row before the chevron slot.
const PAD_X: f32 = 4.0;
/// Gap between the chevron slot, icon, name and detail.
const GAP: f32 = 4.0;
/// Gap between the name and its muted detail (count or units).
const DETAIL_GAP: f32 = 8.0;
/// Chevron slot width; variables reserve it too so icons align with sibling folders.
const CHEVRON: IconSize = IconSize::Xs;
const ICON: IconSize = IconSize::Sm;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    Folder { open: bool },
    Variable,
}

impl RowKind {
    pub fn icon(self) -> Icon {
        match self {
            RowKind::Folder { open: true } => Icon::FolderOpen,
            RowKind::Folder { open: false } => Icon::Folder,
            RowKind::Variable => Icon::VariableDoc,
        }
    }

    pub fn chevron(self) -> Option<Icon> {
        match self {
            RowKind::Folder { open: true } => Some(Icon::ChevronDown),
            RowKind::Folder { open: false } => Some(Icon::ChevronRight),
            RowKind::Variable => None,
        }
    }
}

/// Reserve a full-width, clickable row under a stable `id` (see
/// [`super::nav::row_id`]). Paint it with [`paint_row`] after reacting to the
/// click so the row never lags a frame behind its state.
pub fn allocate_row(ui: &mut Ui, id: Id) -> Response {
    let height = ui.spacing().interact_size.y.max(ICON.px() + 6.0);
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), height));
    let resp = ui.interact(rect, id, Sense::click());
    // egui never focuses on click; do it so the keys continue from the clicked row.
    if resp.clicked() && !resp.has_focus() {
        key_focus::request(ui.ctx(), id);
    }
    if resp.has_focus() {
        key_focus::claim_arrows(ui.ctx(), id);
    }
    if resp.gained_focus() {
        resp.scroll_to_me(None);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Text and icon colors of one row for its hover, focus and selection state.
#[derive(Clone, Copy)]
struct RowColors {
    text: Color32,
    detail: Color32,
    icon: Color32,
    chevron: Color32,
}

/// Paint a row allocated by [`allocate_row`] and describe it for accessibility.
pub fn paint_row(
    ui: &Ui,
    resp: &Response,
    kind: RowKind,
    label: &str,
    detail: &str,
    selected: bool,
) {
    // Built lazily: egui only calls this when accessibility output is wanted,
    // so idle frames don't allocate the label.
    resp.widget_info(|| match kind {
        RowKind::Folder { .. } => {
            egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, label)
        }
        RowKind::Variable => {
            egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, label)
        }
    });

    if !ui.is_rect_visible(resp.rect) {
        return;
    }
    let colors = paint_background(ui, resp, selected);
    let x = paint_icons(ui, resp.rect, kind, colors);
    paint_labels(ui, resp.rect, x, label, detail, colors);
}

/// Selection fill, or hover/focus fill, plus the focus ring; returns the
/// colors for the row's content.
fn paint_background(ui: &Ui, resp: &Response, selected: bool) -> RowColors {
    let visuals = ui.visuals();
    let radius = visuals.widgets.hovered.corner_radius;
    let hot = resp.hovered() || resp.has_focus();
    if selected {
        ui.painter()
            .rect_filled(resp.rect, radius, visuals.selection.bg_fill);
    } else if hot {
        let fill = visuals.widgets.hovered.weak_bg_fill;
        ui.painter().rect_filled(resp.rect, radius, fill);
    }
    if resp.has_focus() {
        key_focus::paint_focus_ring(ui, resp.rect, radius);
    }

    let muted = IconTone::Muted.color(visuals);
    let text = if selected {
        visuals.selection.stroke.color
    } else if hot {
        visuals.strong_text_color()
    } else {
        visuals.text_color()
    };
    RowColors {
        text,
        detail: if selected { text } else { muted },
        icon: if selected {
            text
        } else {
            IconTone::Default.color(visuals)
        },
        chevron: if hot { text } else { muted },
    }
}

/// Chevron slot and kind icon; returns the x where the name starts.
fn paint_icons(ui: &Ui, rect: Rect, kind: RowKind, colors: RowColors) -> f32 {
    let painter = ui.painter();
    let dark = ui.visuals().dark_mode;
    let mut x = rect.left() + PAD_X;
    if let Some(chevron) = kind.chevron() {
        chevron.paint(painter, icon_rect(x, rect, CHEVRON), colors.chevron, dark);
    }
    x += CHEVRON.px() + GAP;
    kind.icon()
        .paint(painter, icon_rect(x, rect, ICON), colors.icon, dark);
    x + ICON.px() + GAP
}

/// Bold name then muted detail from `x`. The detail is laid out first so a
/// long name truncates instead of hiding it.
fn paint_labels(ui: &Ui, rect: Rect, x: f32, label: &str, detail: &str, colors: RowColors) {
    let available = (rect.right() - PAD_X - x).max(0.0);
    let layout = |text: egui::RichText, width: f32| {
        WidgetText::from(text).into_galley(ui, Some(TextWrapMode::Truncate), width, TextStyle::Body)
    };
    let detail_galley =
        (!detail.is_empty()).then(|| layout(egui::RichText::new(detail), available * 0.5));
    let detail_w = detail_galley
        .as_ref()
        .map_or(0.0, |g| g.size().x + DETAIL_GAP);
    let name_galley = layout(
        egui::RichText::new(label).strong(),
        (available - detail_w).max(0.0),
    );

    let center_y = |h: f32| rect.center().y - h * 0.5;
    let name_w = name_galley.size().x;
    let name_pos = pos2(x, center_y(name_galley.size().y));
    ui.painter().galley(name_pos, name_galley, colors.text);
    if let Some(g) = detail_galley {
        let pos = pos2(x + name_w + DETAIL_GAP, center_y(g.size().y));
        ui.painter().galley(pos, g, colors.detail);
    }
}

/// Square icon rect of `size` starting at `x`, vertically centered in `row`.
fn icon_rect(x: f32, row: Rect, size: IconSize) -> Rect {
    let s = size.px();
    Rect::from_min_size(pos2(x, row.center().y - s * 0.5), vec2(s, s))
}
