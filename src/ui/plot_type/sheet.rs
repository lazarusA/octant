//! Review sheet: `cargo test --lib plot_type_picker_sheet -- --ignored`
//! writes `target/icon_sheets/plot_type_picker.png` (dark theme top, light
//! theme bottom): the picker over a gradient standing in for the plot.

use super::picker::show_plot_type_picker;
use crate::plots::PlotType;
use crate::ui::panel_width_tests::plotted;
use crate::ui::test_render::{rasterize, sheet_dir, tessellate_pass, themes, to_image};
use egui::epaint::Mesh;
use egui::{Color32, LayerId, Rect, Shape, pos2, vec2};

const SIZE: egui::Vec2 = vec2(520.0, 240.0);
const PPP: f32 = 2.0;

/// A colorful gradient behind the picker, so its transparency shows.
fn backdrop(ctx: &egui::Context) {
    let rect = Rect::from_min_size(pos2(0.0, 0.0), SIZE);
    let mut mesh = Mesh::default();
    let [a, b] = [
        Color32::from_rgb(40, 90, 200),
        Color32::from_rgb(230, 170, 40),
    ];
    for (corner, color) in [
        (rect.left_top(), a),
        (rect.right_top(), b),
        (rect.right_bottom(), b),
        (rect.left_bottom(), a),
    ] {
        mesh.colored_vertex(corner, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ctx.layer_painter(LayerId::background())
        .add(Shape::mesh(mesh));
}

#[test]
#[ignore = "writes review images to target/icon_sheets"]
fn plot_type_picker_sheet() {
    let (w, h) = ((SIZE.x * PPP) as usize, (SIZE.y * PPP) as usize);
    let mut sheet = image::RgbaImage::new(w as u32, 2 * h as u32);
    for (row, visuals) in themes().iter().enumerate() {
        let (prims, atlas) = tessellate_pass(SIZE, PPP, visuals, |ctx| {
            backdrop(ctx);
            let mut app = plotted(PlotType::Heatmap);
            app.layout.show_plot_picker = true;
            show_plot_type_picker(&mut app, ctx);
        });
        let buf = rasterize(&prims, &atlas, PPP, w, h, visuals.panel_fill);
        let strip = to_image(&buf, w, (0, 0), (w as u32, h as u32), 1);
        image::imageops::replace(&mut sheet, &strip, 0, (row * h) as i64);
    }
    sheet
        .save(sheet_dir().join("plot_type_picker.png"))
        .expect("save sheet");
}
