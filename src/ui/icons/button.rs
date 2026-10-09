//! Toolbar button sized to match the top-bar brand icon.
//!
//! Unlike [`UiIconExt::icon_button`](super::UiIconExt::icon_button), a
//! [`ToolbarButton`] has a fixed height and a larger glyph, and can collapse to
//! an icon-only square whose label moves into the hover tooltip. It has no
//! background at rest; a frame appears only on hover, press or focus, and a
//! muted grey fill marks a button whose popup or panel is open.

use super::Icon;
use super::style::{ICON_GAP, IconSize, IconTone};
use egui::{Galley, Rect, Response, Sense, TextStyle, Ui, Widget, WidgetText, pos2, vec2};
use std::sync::Arc;

/// Height of every top-bar item, equal to the Octant brand icon.
pub const TOOLBAR_ITEM_HEIGHT: f32 = IconSize::Lg.px();

/// Padding around the glyph on every side; with an `Md` glyph this makes the
/// compact button a [`TOOLBAR_ITEM_HEIGHT`] square.
const PAD: f32 = (TOOLBAR_ITEM_HEIGHT - IconSize::Md.px()) * 0.5;
/// Extra padding after the label so text does not touch the frame edge.
const LABEL_PAD_RIGHT: f32 = 5.0;

/// A fixed-height icon button that can collapse to its icon only.
pub struct ToolbarButton<'a> {
    icon: Icon,
    label: &'a str,
    compact: bool,
    hover: Option<&'a str>,
    active: bool,
    owns_popup: bool,
    toggled: bool,
    icon_size: IconSize,
    sense: Sense,
}

impl<'a> ToolbarButton<'a> {
    pub fn new(icon: Icon, label: &'a str) -> Self {
        Self {
            icon,
            label,
            compact: false,
            hover: None,
            active: false,
            owns_popup: false,
            toggled: false,
            icon_size: IconSize::Md,
            sense: Sense::click(),
        }
    }

    /// Hide the label and show it as the hover tooltip instead.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Hover text shown instead of the label tooltip.
    pub fn hover(mut self, text: &'a str) -> Self {
        self.hover = Some(text);
        self
    }

    /// Mark the panel this button toggles as open.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Show as active while the popup attached to this button's response
    /// (via `egui::Popup::from_toggle_button_response` or `menu`) is open.
    pub fn owns_popup(mut self) -> Self {
        self.owns_popup = true;
        self
    }

    /// Make this an on/off switch. While on, it shows an `Accent` tinted fill
    /// and icon that stay visible under hover, so the state is always readable.
    pub fn toggled(mut self, on: bool) -> Self {
        self.toggled = on;
        self
    }

    /// Glyph size; the button stays [`PAD`] larger on every side. Defaults to
    /// `Md`, giving the 24 px toolbar height.
    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = size;
        self
    }

    /// What the button responds to; defaults to clicks. A drag handle senses
    /// `Sense::click_and_drag()`.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Width this button will occupy, without allocating it. Uses the same
    /// layout as drawing, so measured and drawn widths always agree.
    pub fn width(&self, ui: &Ui) -> f32 {
        self.layout(ui).0.x
    }

    /// Button size and, when the label is shown, its galley.
    fn layout(&self, ui: &Ui) -> (egui::Vec2, Option<Arc<Galley>>) {
        let side = self.icon_size.px() + PAD * 2.0;
        let galley = (!self.compact && !self.label.is_empty()).then(|| {
            WidgetText::from(self.label).into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                TextStyle::Button,
            )
        });
        let label_w = galley
            .as_ref()
            .map_or(0.0, |g| ICON_GAP + g.size().x + LABEL_PAD_RIGHT);
        (vec2(side + label_w, side), galley)
    }

    /// Background: an on-switch keeps its accent fill (outlined while
    /// hovered); otherwise the hover / press / focus frame, else the open-state
    /// fill.
    fn paint_frame(&self, ui: &Ui, rect: Rect, response: &Response) {
        let visuals = ui.style().interact(response);
        let interacting =
            response.hovered() || response.is_pointer_button_down_on() || response.has_focus();
        if self.toggled {
            let stroke = if interacting {
                visuals.bg_stroke
            } else {
                egui::Stroke::NONE
            };
            let fill = IconTone::Accent.themed_tint(ui.visuals(), 40, 32);
            ui.painter().rect(
                rect,
                visuals.corner_radius,
                fill,
                stroke,
                egui::StrokeKind::Inside,
            );
        } else if interacting {
            ui.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );
        } else if self.is_active(ui, response) {
            ui.painter().rect_filled(
                rect,
                visuals.corner_radius,
                ui.visuals().widgets.inactive.weak_bg_fill,
            );
        }
    }

    fn is_active(&self, ui: &Ui, response: &Response) -> bool {
        self.active
            || (self.owns_popup
                && egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(response)))
    }
}

impl Widget for ToolbarButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (size, galley) = self.layout(ui);
        let (rect, response) = ui.allocate_exact_size(size, self.sense);

        if ui.is_rect_visible(rect) {
            self.paint_frame(ui, rect, &response);

            let color = if self.toggled {
                IconTone::Accent.color(ui.visuals())
            } else {
                ui.style().interact(&response).text_color()
            };
            let icon_px = self.icon_size.px();
            let icon_rect = Rect::from_min_size(
                pos2(rect.min.x + PAD, rect.center().y - icon_px * 0.5),
                vec2(icon_px, icon_px),
            );
            self.icon
                .paint(ui.painter(), icon_rect, color, ui.visuals().dark_mode);

            if let Some(galley) = galley {
                let text_x = icon_rect.max.x + PAD + ICON_GAP;
                let text_pos = pos2(text_x, rect.center().y - galley.size().y * 0.5);
                ui.painter().galley(text_pos, galley, color);
            }
        }

        match self.hover {
            Some(text) => response.on_hover_text(text),
            None if self.compact => response.on_hover_text(self.label),
            None => response,
        }
    }
}
