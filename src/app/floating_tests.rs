//! A colormap hovered in an overlay's picker (Settings' Layers menu)
//! previews on that overlay's colorbar in the same frame, and the base
//! layer's colorbar keeps its own.

use super::OctantApp;
use crate::app::layers::{LayerId, Source};
use crate::ui::test_input::Harness;
use egui::epaint::{ClippedShape, Mesh};
use egui::{Color32, Rect, Shape, TextureId, Ui, pos2};

/// Frames for areas and popups to finish fading in (half a second).
const SETTLE_FRAMES: usize = 30;
const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 1000.0));

/// One frame the way the app runs it: the preview resets, then the floating
/// panels and colorbars draw.
fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        app.preview_colormap = None;
        app.show_floating_panels(ui.ctx(), SCREEN, true);
    }
}

/// One frame's meshes: colormap swatches (atlas textured) and colorbar bars
/// (vertex colored, many quads).
#[derive(Default)]
struct Painted {
    swatches: Vec<Rect>,
    bars: Vec<(Rect, Vec<Color32>)>,
}

fn bounds(mesh: &Mesh) -> Rect {
    Rect::from_points(&mesh.vertices.iter().map(|v| v.pos).collect::<Vec<_>>())
}

impl Painted {
    fn of(shapes: &[ClippedShape]) -> Self {
        let mut painted = Self::default();
        for clipped in shapes {
            let Shape::Mesh(mesh) = &clipped.shape else {
                continue;
            };
            if mesh.texture_id != TextureId::default() {
                painted.swatches.push(bounds(mesh));
            } else if mesh.vertices.len() >= 256 {
                let colors = mesh.vertices.iter().map(|v| v.color).collect();
                painted.bars.push((bounds(mesh), colors));
            }
        }
        painted
    }

    /// The colors of the bar inside the colorbar panel of `id`.
    fn bar_colors(&self, h: &Harness, id: LayerId) -> Vec<Color32> {
        let panel = h.area(egui::Id::new(("octant_colorbar_overlay", id)));
        self.bars
            .iter()
            .find(|(rect, _)| panel.contains_rect(*rect))
            .map(|(_, colors)| colors.clone())
            .expect("the colorbar paints its bar")
    }
}

#[test]
fn hovering_a_colormap_in_an_overlay_picker_previews_on_its_colorbar() {
    let mut app = OctantApp {
        show_settings_panel: true,
        show_colorbar: true,
        ..Default::default()
    };
    let overlay = app.layers.push(Source::default());
    let mut h = Harness::new(SCREEN);
    // Past the areas' fade-in, so colors compare at full opacity.
    let painted = Painted::of(&h.settle(SETTLE_FRAMES, &mut run(&mut app)));
    let base_before = painted.bar_colors(&h, LayerId::BASE);
    let overlay_before = painted.bar_colors(&h, overlay);
    // The Layers menu lists the topmost overlay first.
    let layer_swatches = painted.swatches;
    let swatch = layer_swatches
        .iter()
        .min_by(|a, b| a.min.y.total_cmp(&b.min.y))
        .expect("the Layers menu shows each layer's swatch")
        .center();
    h.click(swatch, &mut run(&mut app));
    assert_eq!(app.colormaps.target, Some(overlay));
    let mut painted = Painted::of(&h.settle(SETTLE_FRAMES, &mut run(&mut app)));

    // Hover the picker's rows until one other than the overlay's own previews.
    let own = app.layers.get(overlay).map(|l| l.color.colormap);
    let rows: Vec<Rect> = painted
        .swatches
        .iter()
        .filter(|r| !layer_swatches.contains(r) && r.min.y > swatch.y)
        .copied()
        .collect();
    let mut previewed = None;
    for row in rows {
        painted = Painted::of(&h.pointer(row.center(), None, &mut run(&mut app)));
        if app.preview_colormap.is_some() && app.preview_colormap != own {
            previewed = app.preview_colormap;
            break;
        }
    }
    assert!(previewed.is_some(), "a picker row previews a colormap");

    let overlay_now = painted.bar_colors(&h, overlay);
    assert_ne!(
        overlay_now, overlay_before,
        "the overlay's colorbar previews"
    );
    assert_eq!(painted.bar_colors(&h, LayerId::BASE), base_before);
}
