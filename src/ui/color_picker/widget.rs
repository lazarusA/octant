//! Color picker button widget that supports arbitrary custom vector shapes.

use super::popup::show_clean_color_picker_popup;
use super::shape::ColorShape;
use egui::{Color32, Rect, Vec2};

/// A flexible color picker button widget that supports arbitrary custom vector shapes
/// and triggers a sleek, focused color picker popup.
pub struct ShapeColorPicker<'a> {
    id_salt: egui::Id,
    color: &'a mut [f32; 4],
    shape: ColorShape,
    size: Option<Vec2>,
    title: String,
    tooltip: Option<String>,
    anchor_offset: Vec2,
}

impl<'a> ShapeColorPicker<'a> {
    /// Creates a new `ShapeColorPicker` for an RGBA float array `[f32; 4]`.
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        color: &'a mut [f32; 4],
        shape: ColorShape,
    ) -> Self {
        Self {
            id_salt: egui::Id::new(id_salt),
            color,
            shape,
            size: None,
            title: "Color Picker".to_string(),
            tooltip: None,
            anchor_offset: Vec2::new(-70.0, -250.0),
        }
    }

    /// Sets the header title displayed inside the popup.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets an optional hover tooltip on the shape button.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Sets an explicit widget size when rendered with `show()`.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the popup anchor offset relative to `rect.min`.
    pub fn anchor_offset(mut self, offset: Vec2) -> Self {
        self.anchor_offset = offset;
        self
    }

    /// Displays the color picker widget at a specified pre-calculated `Rect`.
    pub fn show_at(self, ui: &mut egui::Ui, rect: Rect) -> egui::Response {
        let resp = ui.interact(rect, self.id_salt.with("btn"), egui::Sense::click());
        let resp = if let Some(tip) = &self.tooltip {
            resp.on_hover_text(tip)
        } else {
            resp
        };

        let style = ui.style();
        let stroke = if resp.hovered() {
            egui::Stroke::new(1.5_f32, style.visuals.widgets.active.fg_stroke.color)
        } else {
            egui::Stroke::new(
                1.0_f32,
                style.visuals.widgets.noninteractive.fg_stroke.color,
            )
        };

        let color_c32 = Color32::from_rgba_unmultiplied(
            (self.color[0] * 255.0).round() as u8,
            (self.color[1] * 255.0).round() as u8,
            (self.color[2] * 255.0).round() as u8,
            (self.color[3] * 255.0).round() as u8,
        );

        // Paint the shape
        self.shape.paint(ui.painter(), rect, color_c32, stroke);

        // Popup open/close state
        let popup_open_id = self.id_salt.with("popup_open");
        let popup_area_id = self.id_salt.with("popup_area");

        if resp.clicked() {
            let is_open = ui
                .data(|d| d.get_temp::<bool>(popup_open_id))
                .unwrap_or(false);
            ui.data_mut(|d| d.insert_temp(popup_open_id, !is_open));
        }

        let popup_pos = rect.min + self.anchor_offset;
        show_clean_color_picker_popup(
            ui,
            self.id_salt,
            self.color,
            popup_open_id,
            popup_area_id,
            popup_pos,
        );

        resp
    }

    /// Allocates layout space and displays the color picker widget.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let size = self.size.unwrap_or(Vec2::splat(16.0));
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::click());
        self.show_at(ui, rect)
    }
}
