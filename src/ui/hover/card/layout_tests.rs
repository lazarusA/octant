//! Layout tests that need an egui context for text measurement.

use super::flow::{FieldRows, KEY_GAP};
use super::layout::{CardLayout, MAX_WIDTH, MIN_WIDTH, PAD};
use super::model::{HoverCard, HoverValue};
use super::tests::sample_fields;
use crate::ui::hover::field::HoverField;
use egui::Color32;

/// Runs `check` inside a frame so it can lay out text.
fn with_ui(check: impl FnMut(&mut egui::Ui)) {
    let ctx = egui::Context::default();
    let mut out = ctx.run_ui(egui::RawInput::default(), check);
    out.textures_delta.clear();
}

fn t2m<'a>(fields: &'a [HoverField], units: &'a str) -> HoverCard<'a> {
    HoverCard {
        title: "2 metre temperature",
        value: HoverValue::Scalar(287.43),
        units,
        swatch: Color32::RED,
        fields,
    }
}

#[test]
fn layout_grows_with_fields_within_width_limits() {
    with_ui(|ui| {
        let fields = sample_fields();
        let with = CardLayout::new(ui.painter(), ui.visuals(), &t2m(&fields, "K"), "287.43");
        let without = CardLayout::new(ui.painter(), ui.visuals(), &t2m(&[], "K"), "287.43");
        assert!(with.size.y > without.size.y);
        assert_eq!(without.fields.rows, 0);
        for l in [with, without] {
            assert!((MIN_WIDTH..=MAX_WIDTH).contains(&l.size.x));
            assert!(l.fields.width <= l.size.x - 2.0 * PAD.x);
        }
    });
}

#[test]
fn long_label_keeps_its_full_width_when_the_pair_fits() {
    with_ui(|ui| {
        let max = MAX_WIDTH - 2.0 * PAD.x;
        let label = "sea_floor_depth_below_geoid_level";
        let fields = [HoverField::new(label, "3 / 10")];
        let rows = FieldRows::layout(ui.painter(), ui.visuals(), &fields, max);
        let natural = |text: &str, font| {
            super::layout::line(ui.painter(), text, font, Color32::WHITE, f32::INFINITY)
                .size()
                .x
        };
        let key_w = natural(label, super::layout::key_font());
        let val_w = natural("3 / 10", super::layout::field_font());
        assert!(
            key_w > max * 0.5,
            "label too short to exercise the old half-width cap"
        );
        // The row is exactly the untruncated pair: nothing was ellipsized.
        assert_eq!(rows.rows, 1);
        assert!((rows.width - (key_w + KEY_GAP + val_w)).abs() < 0.5);
        assert!(rows.width <= max);
    });
}

#[test]
fn oversized_pair_is_truncated_to_one_row() {
    with_ui(|ui| {
        let max = 120.0;
        let fields = [HoverField::new(
            "time",
            "2024-01-15 06:00:00 UTC (forecast step 48)",
        )];
        let rows = FieldRows::layout(ui.painter(), ui.visuals(), &fields, max);
        assert_eq!(rows.rows, 1);
        assert!(rows.width <= max + 0.5, "{} > {max}", rows.width);
    });
}

#[test]
fn units_are_dropped_when_the_value_fills_the_row() {
    with_ui(|ui| {
        let long = "1.2345e-12 1.2345e-12 1.2345e-12 1.2345e-12";
        let card = t2m(&[], "kg m-2 s-1");
        let squeezed = CardLayout::new(ui.painter(), ui.visuals(), &card, long);
        assert!(squeezed.units.is_none());
        let normal = CardLayout::new(ui.painter(), ui.visuals(), &card, "287.43");
        assert!(normal.units.is_some());
        let no_data = HoverCard {
            value: HoverValue::NoData,
            ..t2m(&[], "K")
        };
        let no_data = CardLayout::new(ui.painter(), ui.visuals(), &no_data, "No data");
        assert!(no_data.units.is_none());
    });
}
