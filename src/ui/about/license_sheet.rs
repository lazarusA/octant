//! Review sheet: `cargo test --lib license_viewer_contact_sheet -- --ignored`
//! writes `target/icon_sheets/license_viewer.png` (dark theme left, light
//! theme right): the colormap and third-party license viewers as in About.

use crate::ui::test_render::{rasterize, sheet_dir, tessellate_pass, themes, to_image};

const SIZE: egui::Vec2 = egui::vec2(560.0, 760.0);
const PPP: f32 = 2.0;

#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn license_viewer_contact_sheet() {
    let (w, h) = ((SIZE.x * PPP) as usize, (SIZE.y * PPP) as usize);
    let mut sheet = image::RgbaImage::new(2 * w as u32, h as u32);
    for (column, visuals) in themes().iter().enumerate() {
        let (prims, atlas) = tessellate_pass(SIZE, PPP, visuals, |ctx| {
            egui::Area::new(egui::Id::new("license_sheet"))
                .fixed_pos(egui::pos2(12.0, 12.0))
                .fade_in(false)
                .show(ctx, |ui| {
                    ui.set_width(SIZE.x - 24.0);
                    super::licenses::show_colormap_licenses(ui);
                    ui.add_space(12.0);
                    super::licenses::show_document(
                        ui,
                        "third_party",
                        super::licenses::THIRD_PARTY_LICENSES,
                        "https://example.invalid",
                    );
                });
        });
        let bg = visuals.panel_fill;
        let buf = rasterize(&prims, &atlas, PPP, w, h, bg);
        let image = to_image(&buf, w, (0, 0), (w as u32, h as u32), 1);
        image::imageops::overlay(&mut sheet, &image, (column * w) as i64, 0);
    }
    let path = sheet_dir().join("license_viewer.png");
    sheet.save(&path).expect("save sheet");
    println!("wrote {}", path.display());
}
