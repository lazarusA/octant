//! Review sheet: `cargo test --lib toast_contact_sheet -- --ignored` writes
//! `target/icon_sheets/toasts.png` (dark theme top, light theme bottom): an
//! info, a merged warning, an error with a long detail and a saved figure.

use std::path::PathBuf;

use egui::{Rect, pos2, vec2};

use super::model::{Notice, Severity, ToastAction};
use super::queue::Toasts;
use crate::ui::test_render::theme_sheet;

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
    theme_sheet("toasts.png", SIZE, PPP, |ctx| {
        let mut toasts = sample();
        let canvas = Rect::from_min_size(pos2(0.0, 0.0), SIZE);
        super::paint::show(ctx, &mut toasts, canvas, web_time::Instant::now());
    });
}
