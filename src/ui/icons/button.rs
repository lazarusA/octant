//! Toolbar button sized to match the top-bar brand icon.
//!
//! Unlike [`UiIconExt::icon_button`](super::UiIconExt::icon_button), a
//! [`ToolbarButton`] has a fixed height and a larger glyph, and can collapse to
//! an icon-only square whose label moves into the hover tooltip. It has no
//! background at rest; a frame appears only on hover, press or focus, and a
//! muted grey fill marks a button whose popup or panel is open.

use super::Icon;
use egui::{Rect, Response, Sense, TextStyle, Ui, Widget, WidgetText, pos2, vec2};

/// Height of every top-bar item, equal to the Octant brand icon.
pub const TOOLBAR_ITEM_HEIGHT: f32 = 24.0;
/// Glyph size drawn inside a toolbar button.
pub const TOOLBAR_ICON_SIZE: f32 = 18.0;

/// Horizontal padding around the glyph; makes the compact button a square.
const PAD_X: f32 = (TOOLBAR_ITEM_HEIGHT - TOOLBAR_ICON_SIZE) * 0.5;
/// Gap between glyph and label.
const GAP: f32 = 5.0;
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

    /// Width the button will occupy, without allocating it.
    pub fn width(ui: &Ui, label: &str, compact: bool) -> f32 {
        if compact || label.is_empty() {
            return TOOLBAR_ITEM_HEIGHT;
        }
        let galley = WidgetText::from(label).into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Button,
        );
        TOOLBAR_ITEM_HEIGHT + GAP + galley.size().x + LABEL_PAD_RIGHT
    }
}

impl Widget for ToolbarButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let show_label = !self.compact && !self.label.is_empty();
        let galley = show_label.then(|| {
            WidgetText::from(self.label).into_galley(
                ui,
                Some(egui::TextWrapMode::Extend),
                f32::INFINITY,
                TextStyle::Button,
            )
        });
        let label_w = galley
            .as_ref()
            .map_or(0.0, |g| GAP + g.size().x + LABEL_PAD_RIGHT);

        let size = vec2(TOOLBAR_ITEM_HEIGHT + label_w, TOOLBAR_ITEM_HEIGHT);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let active = self.active
                || (self.owns_popup
                    && egui::Popup::is_id_open(
                        ui.ctx(),
                        egui::Popup::default_response_id(&response),
                    ));
            // Frameless at rest; the frame only signals hover, press or focus,
            // and a muted grey fill marks an open popup or panel.
            let interacting =
                response.hovered() || response.is_pointer_button_down_on() || response.has_focus();
            if active && !interacting {
                ui.painter().rect_filled(
                    rect,
                    visuals.corner_radius,
                    ui.visuals().widgets.inactive.weak_bg_fill,
                );
            } else if interacting {
                ui.painter().rect(
                    rect,
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
            }

            let color = visuals.text_color();
            let icon_rect = Rect::from_min_size(
                pos2(
                    rect.min.x + PAD_X,
                    rect.center().y - TOOLBAR_ICON_SIZE * 0.5,
                ),
                vec2(TOOLBAR_ICON_SIZE, TOOLBAR_ICON_SIZE),
            );
            self.icon
                .paint(ui.painter(), icon_rect, color, ui.visuals().dark_mode);

            if let Some(galley) = galley {
                let text_pos = pos2(
                    icon_rect.max.x + PAD_X + GAP,
                    rect.center().y - galley.size().y * 0.5,
                );
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
