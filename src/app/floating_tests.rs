//! A colormap hovered in an overlay's picker (opened by clicking its
//! colorbar) previews on that overlay's colorbar in the same frame, and the
//! base layer's colorbar keeps its own.

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

    /// The bar inside the colorbar panel of `id`: its rect and colors.
    fn bar(&self, h: &Harness, id: LayerId) -> &(Rect, Vec<Color32>) {
        let panel = h.area(egui::Id::unique(("octant_colorbar_overlay", id)));
        self.bars
            .iter()
            .find(|(rect, _)| panel.contains_rect(*rect))
            .expect("the colorbar paints its bar")
    }

    fn bar_rect(&self, h: &Harness, id: LayerId) -> Rect {
        self.bar(h, id).0
    }

    fn bar_colors(&self, h: &Harness, id: LayerId) -> Vec<Color32> {
        self.bar(h, id).1.clone()
    }
}

#[test]
fn hovering_a_colormap_in_an_overlay_picker_previews_on_its_colorbar() {
    // Other tests add custom maps, which would change the colors compared.
    let _registry = crate::utils::colormap::registry::test_lock();
    let mut app = OctantApp::default();
    app.layout.show_settings_panel = true;
    let overlay = app.layers.push(Source::default());
    let mut h = Harness::new(SCREEN);
    // Past the areas' fade-in, so colors compare at full opacity.
    let painted = Painted::of(&h.settle(SETTLE_FRAMES, &mut run(&mut app)));
    let base_before = painted.bar_colors(&h, LayerId::BASE);
    let overlay_before = painted.bar_colors(&h, overlay);
    let bar = painted.bar_rect(&h, overlay);
    // Swatches painted before the picker opened are not its rows.
    let earlier = painted.swatches;
    h.click(bar.center(), &mut run(&mut app));
    assert_eq!(app.colormaps.target, Some(overlay));
    let mut painted = Painted::of(&h.settle(SETTLE_FRAMES, &mut run(&mut app)));

    // Hover the picker's rows until one other than the overlay's own previews.
    let own = app.layers.get(overlay).map(|l| l.color.colormap);
    let rows: Vec<Rect> = painted
        .swatches
        .iter()
        .filter(|r| !earlier.contains(r))
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
