//! Review sheet: `cargo test --lib hover_card_contact_sheet -- --ignored` writes
//! `target/icon_sheets/hover_card.png` (dark theme top, light theme bottom). Each theme
//! is four 380x300 pt canvases whose hover points send the card to a different corner,
//! with scalar, no-data, and band-composite values.

use super::model::{HoverCard, HoverValue};
use super::tests::sample_fields;
use super::{Anchoring, show_card};
use crate::ui::hover::composite::CompositeKind;
use crate::ui::hover::field::HoverField;
use crate::ui::test_render::{rasterize, sheet_dir, tessellate_pass, themes, to_image};
use egui::{Color32, Id, LayerId, Order, Pos2, Rect, pos2, vec2};

const QUADRANT: egui::Vec2 = vec2(380.0, 300.0);
const PPP: f32 = 2.0;

fn paint_background(ctx: &egui::Context, size: egui::Vec2) {
    let bg = ctx.layer_painter(LayerId::new(Order::Background, Id::new("bg")));
    for i in 0..19 {
        let t = i as f32 / 18.0;
        let c = Color32::from_rgb((40.0 + 200.0 * t) as u8, 90, (220.0 - 180.0 * t) as u8);
        let column = Rect::from_min_size(pos2(i as f32 * 40.0, 0.0), vec2(40.0, size.y));
        bg.rect_filled(column, 0, c);
    }
}

fn card<'a>(
    value: HoverValue,
    title: &'a str,
    units: &'a str,
    fields: &'a [HoverField],
) -> HoverCard<'a> {
    HoverCard {
        title,
        value,
        units,
        swatch: Color32::from_rgb(230, 120, 60),
        fields,
    }
}

#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn hover_card_contact_sheet() {
    let size = QUADRANT * 2.0;
    let (w, h) = ((size.x * PPP) as usize, (size.y * PPP) as usize);
    let mut sheet = image::RgbaImage::new(w as u32, 2 * h as u32);
    for (row, visuals) in themes().iter().enumerate() {
        let (prims, atlas) = tessellate_pass(size, PPP, visuals, |ctx| {
            paint_background(ctx, size);
            let fields = sample_fields();
            let t2m = card(
                HoverValue::Scalar(287.43),
                "2 metre temperature",
                "K",
                &fields[..],
            );
            let nan = card(HoverValue::NoData, "sst", "K", &fields[1..]);
            let bands = [
                HoverField::new("R", "NIR 0.312"),
                HoverField::new("G", "Red 0.041"),
                HoverField::new("B", "Green 0.063"),
            ];
            let false_color = HoverValue::Composite(CompositeKind::FalseColor);
            let rgb = card(false_color, "Multi-band raster", "", &bands);
            // (quadrant origin, hover point within it, card)
            let scenes: [(Pos2, Pos2, &HoverCard); 4] = [
                (pos2(0.0, 0.0), pos2(40.0, 40.0), &t2m),
                (pos2(QUADRANT.x, 0.0), pos2(340.0, 260.0), &t2m),
                (pos2(0.0, QUADRANT.y), pos2(330.0, 50.0), &nan),
                (pos2(QUADRANT.x, QUADRANT.y), pos2(60.0, 250.0), &rgb),
            ];
            for (origin, local, card) in scenes {
                let canvas = Rect::from_min_size(origin, QUADRANT);
                let target = origin + local.to_vec2();
                show_card(
                    ctx,
                    canvas,
                    target,
                    Some(target),
                    Anchoring::Connected,
                    card,
                );
            }
        });
        let buf = rasterize(&prims, &atlas, PPP, w, h, visuals.panel_fill);
        let strip = to_image(&buf, w, (0, 0), (w as u32, h as u32), 1);
        image::imageops::replace(&mut sheet, &strip, 0, (row * h) as i64);
    }
    sheet
        .save(sheet_dir().join("hover_card.png"))
        .expect("save sheet");
}
