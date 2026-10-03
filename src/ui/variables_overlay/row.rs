//! Full-width clickable tree row shared by folders and variables:
//! `[chevron] [icon] name  detail`, where every pixel of the row is a hit target.

use crate::ui::icons::{Icon, IconSize, IconTone};
use egui::{Rect, Response, Sense, TextStyle, TextWrapMode, Ui, WidgetText, pos2, vec2};

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

/// Reserve a full-width, clickable row. Paint it with [`paint_row`] after
/// reacting to the click so the row never lags a frame behind its state.
pub fn allocate_row(ui: &mut Ui) -> Response {
    let height = ui.spacing().interact_size.y.max(ICON.px() + 6.0);
    let size = vec2(ui.available_width(), height);
    let (_, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
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
    let widget_info = match kind {
        RowKind::Folder { .. } => {
            egui::WidgetInfo::labeled(egui::WidgetType::CollapsingHeader, true, label)
        }
        RowKind::Variable => {
            egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, label)
        }
    };
    resp.widget_info(|| widget_info.clone());

    let rect = resp.rect;
    if !ui.is_rect_visible(rect) {
        return;
    }
    let visuals = ui.visuals();
    let painter = ui.painter();

    let hot = resp.hovered() || resp.has_focus();
    if selected {
        painter.rect_filled(
            rect,
            visuals.widgets.hovered.corner_radius,
            visuals.selection.bg_fill,
        );
    } else if hot {
        let w = &visuals.widgets.hovered;
        painter.rect_filled(rect, w.corner_radius, w.weak_bg_fill);
    }
    if resp.has_focus() {
        let w = &visuals.widgets.hovered;
        painter.rect_stroke(rect, w.corner_radius, w.bg_stroke, egui::StrokeKind::Inside);
    }

    let text_color = if selected {
        visuals.selection.stroke.color
    } else if hot {
        visuals.strong_text_color()
    } else {
        visuals.text_color()
    };
    let muted = IconTone::Muted.color(visuals);
    let dark = visuals.dark_mode;

    let mut x = rect.left() + PAD_X;
    if let Some(chevron) = kind.chevron() {
        let r = icon_rect(x, rect, CHEVRON);
        chevron.paint(painter, r, if hot { text_color } else { muted }, dark);
    }
    x += CHEVRON.px() + GAP;

    let icon_color = if selected {
        text_color
    } else {
        IconTone::Default.color(visuals)
    };
    kind.icon()
        .paint(painter, icon_rect(x, rect, ICON), icon_color, dark);
    x += ICON.px() + GAP;

    // Lay out the detail first so a long name truncates instead of hiding it.
    let available = (rect.right() - PAD_X - x).max(0.0);
    let detail_galley = (!detail.is_empty()).then(|| {
        WidgetText::from(egui::RichText::new(detail)).into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            available * 0.5,
            TextStyle::Body,
        )
    });
    let detail_w = detail_galley
        .as_ref()
        .map_or(0.0, |g| g.size().x + DETAIL_GAP);
    let name_galley = WidgetText::from(egui::RichText::new(label).strong()).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        (available - detail_w).max(0.0),
        TextStyle::Body,
    );

    let name_pos = pos2(x, rect.center().y - name_galley.size().y * 0.5);
    let name_w = name_galley.size().x;
    painter.galley(name_pos, name_galley, text_color);

    if let Some(g) = detail_galley {
        let pos = pos2(x + name_w + DETAIL_GAP, rect.center().y - g.size().y * 0.5);
        let color = if selected { text_color } else { muted };
        painter.galley(pos, g, color);
    }
}

/// Square icon rect of `size` starting at `x`, vertically centered in `row`.
fn icon_rect(x: f32, row: Rect, size: IconSize) -> Rect {
    let s = size.px();
    Rect::from_min_size(pos2(x, row.center().y - s * 0.5), vec2(s, s))
}
