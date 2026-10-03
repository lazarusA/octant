use super::wordmark::{COLS, Metrics, ROWS, WORD};
use super::{FACE_SHADES, Wordmark, face_colors};
use egui::Visuals;

#[test]
fn letters_are_five_by_five_and_non_empty() {
    for glyph in WORD {
        assert!(
            glyph.iter().all(|row| *row < 1 << 5),
            "row wider than 5 cells"
        );
        assert!(glyph.iter().any(|row| *row != 0), "empty letter");
    }
    assert_eq!(COLS, 35);
}

#[test]
fn metrics_snap_to_whole_pixels_and_pick_style() {
    for ppp in [1.0_f32, 1.5, 2.0] {
        for cell in [5.0_f32, 6.0, 7.0] {
            let m = Metrics::new(cell, ppp);
            let px = m.cell * ppp;
            assert!((px - px.round()).abs() < 1e-4, "cell not on whole pixels");
            assert_eq!(m.blocks, px >= 7.0);
            let size = m.size();
            assert!((size.x - (COLS as f32 * m.cell + m.depth)).abs() < 1e-4);
            assert!((size.y - (ROWS as f32 * m.cell + m.depth)).abs() < 1e-4);
        }
    }
}

#[test]
fn side_faces_differ_from_front_in_both_themes() {
    for visuals in [Visuals::dark(), Visuals::light()] {
        let faces = face_colors(&visuals, visuals.strong_text_color());
        assert_ne!(faces[0], faces[1]);
        assert_ne!(faces[1], faces[2]);
    }
    assert_eq!(FACE_SHADES[0], 1.0);
}

#[test]
fn wordmark_paints_at_every_size_and_theme() {
    let ctx = egui::Context::default();
    for visuals in [Visuals::dark(), Visuals::light()] {
        ctx.set_visuals(visuals);
        for cell in [5.0, 6.0, 7.0] {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                let r = ui.add(Wordmark::new(cell));
                assert!(r.rect.width() > r.rect.height() * 5.0);
            });
            out.textures_delta.clear();
        }
    }
}

/// Review sheet: `cargo test --lib wordmark_contact_sheet -- --ignored` writes
/// `target/icon_sheets/wordmark.png` (5/6/7 pt cells at 1x and 2x; dark theme
/// left, light theme right).
#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn wordmark_contact_sheet() {
    use crate::ui::test_render::{rasterize, sheet_dir, tessellate_pass, themes, to_image};
    use egui::{pos2, vec2};
    const CASES: [(f32, f32); 6] = [
        (5.0, 1.0),
        (6.0, 1.0),
        (7.0, 1.0),
        (5.0, 2.0),
        (6.0, 2.0),
        (7.0, 2.0),
    ];
    let (w_pt, h_pt) = (300.0_f32, 60.0_f32);
    let mut sheet = image::RgbaImage::new(1200, 720);
    for (col, visuals) in themes().iter().enumerate() {
        for (row, &(cell, ppp)) in CASES.iter().enumerate() {
            let (prims, atlas) = tessellate_pass(vec2(w_pt, h_pt), ppp, visuals, |ctx| {
                egui::Area::new(egui::Id::new("wordmark"))
                    .fixed_pos(pos2(12.0, 12.0))
                    .fade_in(false)
                    .show(ctx, |ui| ui.add(Wordmark::new(cell)));
            });
            let (w, h) = ((w_pt * ppp) as usize, (h_pt * ppp) as usize);
            let buf = rasterize(&prims, &atlas, ppp, w, h, visuals.panel_fill);
            let strip = to_image(&buf, w, (0, 0), (600, 120), (2.0 / ppp) as usize);
            image::imageops::replace(&mut sheet, &strip, col as i64 * 600, row as i64 * 120);
        }
    }
    sheet
        .save(sheet_dir().join("wordmark.png"))
        .expect("save sheet");
}
