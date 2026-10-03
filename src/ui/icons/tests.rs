use super::sheet;
use super::style::stroke_width;
use super::*;
use egui::{Color32, Painter, Rect, Visuals, vec2};

#[test]
fn test_all_icons_unique_and_non_empty() {
    let mut names = std::collections::HashSet::new();
    for &icon in Icon::ALL {
        let name = icon.name();
        assert!(!name.is_empty(), "Icon name cannot be empty");
        assert!(
            names.insert(name),
            "Duplicate icon name '{name}' found in Icon::ALL"
        );
        assert!(
            !icon.category().is_empty(),
            "Category for '{name}' cannot be empty"
        );
    }
    assert_eq!(Icon::ALL.len(), 48, "Expected 48 total procedural icons");
}

#[test]
fn test_icon_painting_every_size_and_theme() {
    let ctx = egui::Context::default();
    for size in IconSize::ALL {
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), vec2(size.px(), size.px()));
        for &icon in Icon::ALL {
            for is_dark in [true, false] {
                let painter = Painter::new(ctx.clone(), egui::LayerId::background(), rect);
                let color = if is_dark {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };
                icon.paint(&painter, rect, color, is_dark);
            }
        }
    }
}

#[test]
fn test_size_steps_increase_with_stroke_weight() {
    for pair in IconSize::ALL.windows(2) {
        assert!(pair[0].px() < pair[1].px());
        assert!(stroke_width(pair[0].px()) < stroke_width(pair[1].px()));
    }
}

/// WCAG relative luminance of an sRGB color.
fn luminance(c: Color32) -> f32 {
    let lin = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[test]
fn test_tones_meet_non_text_contrast_in_both_themes() {
    for visuals in [Visuals::dark(), Visuals::light()] {
        for tone in IconTone::ALL {
            let ratio = contrast(tone.color(&visuals), visuals.panel_fill);
            assert!(
                ratio >= 3.0,
                "{} on {} panel: contrast {ratio:.2} < 3.0",
                tone.name(),
                if visuals.dark_mode { "dark" } else { "light" }
            );
        }
    }
}

#[test]
fn test_tone_rgb_matches_visuals_color() {
    for visuals in [Visuals::dark(), Visuals::light()] {
        for tone in IconTone::ALL {
            if let Some(rgb) = tone.rgb(visuals.dark_mode) {
                assert_eq!(rgb, tone.color(&visuals), "{} drifted", tone.name());
            }
        }
    }
}

/// Every icon's tessellated geometry stays inside its box at every size and
/// scale, which also catches NaN vertices and runaway miter spikes.
#[test]
fn every_icon_stays_inside_its_box() {
    for &(size, ppp) in &sheet::COLUMNS {
        for &icon in Icon::ALL {
            let (prims, _) = sheet::tessellate(&[icon], size, ppp, &Visuals::dark());
            // Allow one point of anti-aliasing feather around the box.
            let bounds = sheet::tile_rect(0, size).expand(1.0);
            for prim in &prims {
                let egui::epaint::Primitive::Mesh(mesh) = &prim.primitive else {
                    continue;
                };
                for v in mesh.vertices.iter().filter(|v| v.color.a() > 0) {
                    assert!(
                        bounds.contains(v.pos),
                        "{} at {size}px x{ppp}: vertex {:?} outside {bounds:?}",
                        icon.name(),
                        v.pos
                    );
                }
            }
        }
    }
}
