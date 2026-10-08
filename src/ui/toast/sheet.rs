//! Review sheet: `cargo test --lib toast_contact_sheet -- --ignored` writes
//! `target/icon_sheets/toasts.png` (dark theme top, light theme bottom): an
//! info, a merged warning, an error with a long detail and a saved figure.

use std::path::PathBuf;

use egui::{Rect, pos2, vec2};

use super::model::{Notice, Severity, ToastAction};
use super::queue::Toasts;
use crate::ui::test_render::{rasterize, sheet_dir, tessellate_pass, themes, to_image};

const SIZE: egui::Vec2 = vec2(420.0, 360.0);
const PPP: f32 = 2.0;

fn sample() -> Toasts {
    let mut toasts = Toasts::default();
    toasts.push(Notice::new(Severity::Info, "Loading coastlines", ""));
    for _ in 0..3 {
        toasts.push(Notice::new(
            Severity::Warning,
            "Coordinates unavailable",
            "Coordinates of 't2m' failed to load: the plot falls back to indices",
        ));
    }
    toasts.push(Notice::new(
        Severity::Error,
        "Couldn't open dataset",
        "Failed to read IFD metadata for 'off_luv24.tif': IFD 0: unsupported TIFF: \
         SGI LogLuv (HDR) photometric interpretation",
    ));
    toasts.push(
        Notice::new(Severity::Success, "Saved", "octant_figure.png").with_action(
            ToastAction::RevealFile(PathBuf::from("/tmp/octant_figure.png")),
        ),
    );
    toasts
}

#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn toast_contact_sheet() {
    let (w, h) = ((SIZE.x * PPP) as usize, (SIZE.y * PPP) as usize);
    let mut sheet = image::RgbaImage::new(w as u32, 2 * h as u32);
    for (row, visuals) in themes().iter().enumerate() {
        let (prims, atlas) = tessellate_pass(SIZE, PPP, visuals, |ctx| {
            let mut toasts = sample();
            let canvas = Rect::from_min_size(pos2(0.0, 0.0), SIZE);
            super::paint::show(ctx, &mut toasts, canvas, web_time::Instant::now());
        });
        let buf = rasterize(&prims, &atlas, PPP, w, h, visuals.panel_fill);
        let strip = to_image(&buf, w, (0, 0), (w as u32, h as u32), 1);
        image::imageops::replace(&mut sheet, &strip, 0, (row * h) as i64);
    }
    sheet
        .save(sheet_dir().join("toasts.png"))
        .expect("save sheet");
}
