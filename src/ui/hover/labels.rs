//! Hover readings of label images: the label id under the cursor, its
//! color and its properties.

use crate::app::layers::Layer;
use crate::ui::hover::card::HoverValue;
use egui::Color32;

/// `(value, swatch, properties)` of `layer` at raw sample `raw`, when the
/// layer shows a label image. Label ids are stored as the bits of `raw`;
/// label 0 is the background, read as no data.
pub fn label_reading(layer: &Layer, raw: f32) -> Option<(HoverValue, Color32, &str)> {
    let style = layer.labels()?;
    let id = raw.to_bits();
    let Some([r, g, b, a]) = crate::plots::labels::label_color(&style.info, id) else {
        return Some((HoverValue::NoData, Color32::TRANSPARENT, ""));
    };
    let alpha = (f32::from(a) * layer.color.opacity)
        .round()
        .clamp(0.0, 255.0) as u8;
    let swatch = Color32::from_rgba_unmultiplied(r, g, b, alpha);
    let properties = style.info.properties_line(u64::from(id)).unwrap_or("");
    Some((HoverValue::Label(id), swatch, properties))
}
