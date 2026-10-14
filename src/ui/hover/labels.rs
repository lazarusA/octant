//! Hover readings of label images: the label id under the cursor, its
//! color and its properties.

use crate::app::layers::Layer;
use crate::ui::hover::card::HoverValue;
use crate::ui::hover::composite::CompositeKind;
use egui::Color32;

/// `(value, swatch, units)` of `layer` at raw sample `raw`: a label image's
/// id, color and properties, else the value (as `composite` when drawn as
/// one) in the color `swatch` gives, with the variable's `units`.
pub fn layer_reading<'a>(
    layer: &'a Layer,
    raw: f32,
    composite: Option<CompositeKind>,
    units: &'a str,
    swatch: impl FnOnce() -> Color32,
) -> (HoverValue, Color32, &'a str) {
    label_reading(layer, raw)
        .unwrap_or_else(|| (HoverValue::from_raw(raw, composite), swatch(), units))
}

/// `(value, swatch, properties)` of `layer` at raw sample `raw`, when the
/// layer shows a label image. Label ids are stored as the bits of `raw`;
/// label 0 is the background, read as no data.
fn label_reading(layer: &Layer, raw: f32) -> Option<(HoverValue, Color32, &str)> {
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
