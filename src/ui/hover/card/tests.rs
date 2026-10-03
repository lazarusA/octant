use super::flow::{FlowSlot, SEPARATOR, flow};
use super::layout::CORNER_RADIUS;
use super::leader::{leader_anchor, leader_elbow};
use super::model::{HoverCard, HoverValue};
use super::place::{EDGE_MARGIN, Side, card_bounds, place_connected, place_following};
use crate::ui::hover::field::HoverField;
use egui::{Rect, pos2, vec2};

const CANVAS: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));
const CARD: egui::Vec2 = vec2(200.0, 110.0);

fn fmt(v: HoverValue) -> String {
    let mut buf = [0u8; 32];
    v.format(&mut buf).to_owned()
}

#[test]
fn values_format_compactly() {
    assert_eq!(fmt(HoverValue::Scalar(287.43)), "287.43");
    assert_eq!(fmt(HoverValue::Scalar(101_325.0)), "101325");
    assert_eq!(fmt(HoverValue::Scalar(0.0123)), "0.0123");
    assert_eq!(fmt(HoverValue::Scalar(2.0)), "2");
    assert_eq!(fmt(HoverValue::Scalar(-0.00001)), "-1e-5");
    assert_eq!(fmt(HoverValue::Scalar(1.25e7)), "1.25e7");
    assert_eq!(fmt(HoverValue::Scalar(-0.00004)), "-4e-5");
    assert_eq!(fmt(HoverValue::Scalar(0.0)), "0");
}

#[test]
fn raw_samples_classify() {
    assert_eq!(HoverValue::from_raw(f32::NAN, false), HoverValue::NoData);
    assert_eq!(HoverValue::from_raw(f32::NAN, true), HoverValue::NoData);
    let packed = (12 | (200 << 8) | (34 << 16)) as f32;
    assert_eq!(
        HoverValue::from_raw(packed, true),
        HoverValue::Rgb([12, 200, 34])
    );
    assert_eq!(fmt(HoverValue::NoData), "No data");
    assert_eq!(fmt(HoverValue::Rgb([12, 200, 34])), "12, 200, 34");
    assert!(!HoverValue::NoData.shows_units());
    // Infinities are real values, not missing data.
    assert_eq!(
        HoverValue::from_raw(f32::INFINITY, false),
        HoverValue::Scalar(f32::INFINITY)
    );
    assert_eq!(fmt(HoverValue::Scalar(f32::NEG_INFINITY)), "-inf");
}

#[test]
fn long_name_becomes_title() {
    assert_eq!(
        HoverCard::title_for("t2m", Some(" 2 metre temperature ")),
        "2 metre temperature"
    );
    assert_eq!(HoverCard::title_for("t2m", Some(" ")), "t2m");
    assert_eq!(HoverCard::title_for("t2m", None), "t2m");
}

#[test]
fn connected_card_prefers_right_and_flips_left_near_edge() {
    let right = place_connected(pos2(200.0, 300.0), CARD, CANVAS);
    assert_eq!(right.side, Side::Right);
    let left = place_connected(pos2(700.0, 300.0), CARD, CANVAS);
    assert_eq!(left.side, Side::Left);
    assert!(left.rect.right() < 700.0);
}

#[test]
fn connected_card_stays_inside_and_never_covers_target() {
    let inner = CANVAS.shrink(EDGE_MARGIN);
    for x in (0..=800).step_by(25) {
        for y in (0..=600).step_by(25) {
            let target = pos2(x as f32, y as f32);
            let p = place_connected(target, CARD, CANVAS);
            assert!(inner.contains_rect(p.rect), "{target:?} -> {:?}", p.rect);
            assert!(
                !p.rect.contains(target),
                "{target:?} covered by {:?}",
                p.rect
            );
        }
    }
}

#[test]
fn leader_ends_under_the_facing_edge_on_every_side() {
    for target in [pos2(200.0, 300.0), pos2(700.0, 300.0), pos2(400.0, 4.0)] {
        let p = place_connected(target, CARD, CANVAS);
        let anchor = leader_anchor(target, &p);
        assert!(
            p.rect.contains(anchor),
            "{:?}: {anchor:?} outside card",
            p.side
        );
        let edge_gap = match p.side {
            Side::Right => anchor.x - p.rect.left(),
            Side::Left => p.rect.right() - anchor.x,
            Side::Below => anchor.y - p.rect.top(),
            Side::Above => p.rect.bottom() - anchor.y,
        };
        assert!((0.0..=f32::from(CORNER_RADIUS)).contains(&edge_gap));
    }
}

#[test]
fn arm_bends_once_and_points_toward_canvas_centre() {
    for target in [
        pos2(150.0, 120.0),
        pos2(650.0, 120.0),
        pos2(150.0, 480.0),
        pos2(650.0, 480.0),
    ] {
        let p = place_connected(target, CARD, CANVAS);
        let anchor = leader_anchor(target, &p);
        let elbow = leader_elbow(target, anchor, p.side);
        // Vertical leg from the point, horizontal leg into the card.
        assert_eq!(elbow.x, target.x);
        assert_eq!(elbow.y, anchor.y);
        assert!(
            (elbow.y - target.y).abs() > 18.0,
            "{target:?}: arm does not bend"
        );
        // Card sits on the canvas-centre side of the point on both axes.
        let toward = p.rect.center() - target;
        let centre = CANVAS.center() - target;
        assert!(
            toward.x * centre.x > 0.0 && toward.y * centre.y > 0.0,
            "{target:?}"
        );
    }
}

fn slots(widths: &[f32], max: f32) -> Vec<FlowSlot> {
    flow(widths.iter().copied(), max).collect()
}

#[test]
fn pairs_share_a_row_with_dividers_between() {
    let s = slots(&[50.0, 50.0, 40.0], 200.0);
    assert_eq!(
        s[0],
        FlowSlot {
            x: 0.0,
            row: 0,
            divided: false
        }
    );
    assert_eq!(
        s[1],
        FlowSlot {
            x: 50.0 + SEPARATOR,
            row: 0,
            divided: true
        }
    );
    assert_eq!(s[2].row, 0);
    assert!(s[2].divided);
}

#[test]
fn pairs_wrap_without_a_leading_divider() {
    let s = slots(&[100.0, 100.0, 60.0], 200.0);
    assert_eq!(
        s[1],
        FlowSlot {
            x: 0.0,
            row: 1,
            divided: false
        }
    );
    // The third pair still fits after the second on the new row.
    assert_eq!(
        s[2],
        FlowSlot {
            x: 100.0 + SEPARATOR,
            row: 1,
            divided: true
        }
    );
}

#[test]
fn exact_fit_stays_on_the_row_and_oversized_pairs_get_their_own() {
    let exact = slots(&[90.0, 200.0 - 90.0 - SEPARATOR], 200.0);
    assert_eq!(exact[1].row, 0);
    let wide = slots(&[300.0, 20.0, 20.0], 200.0);
    assert_eq!(wide.iter().map(|s| s.row).collect::<Vec<_>>(), [0, 1, 1]);
    assert!(slots(&[], 200.0).is_empty());
}

#[test]
fn following_card_flips_near_bottom_right() {
    let r = place_following(pos2(790.0, 590.0), CARD, CANVAS);
    assert!(r.right() < 790.0 && r.bottom() < 590.0);
    assert!(CANVAS.contains_rect(r));
}

#[test]
fn small_canvas_falls_back_to_the_viewport() {
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1200.0, 800.0));
    let narrow = Rect::from_min_size(pos2(1050.0, 0.0), vec2(150.0, 800.0));
    assert_eq!(card_bounds(CANVAS, viewport, CARD), CANVAS);
    assert_eq!(card_bounds(narrow, viewport, CARD), viewport);
    // A canvas partly off screen only counts its visible part.
    let offscreen = CANVAS.translate(vec2(1100.0, 0.0));
    assert_eq!(card_bounds(offscreen, viewport, CARD), viewport);
}

#[test]
fn oversized_card_does_not_panic() {
    let tiny = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 40.0));
    let p = place_connected(pos2(25.0, 20.0), CARD, tiny);
    assert_eq!(p.rect.size(), CARD);
}

pub(super) fn sample_fields() -> Vec<HoverField> {
    vec![
        HoverField::new("time", "2024-01-15 06:00"),
        HoverField::new("latitude", "45.50°N"),
        HoverField::new("longitude", "12.25°W"),
        HoverField::new("level", "850 hPa"),
    ]
}
