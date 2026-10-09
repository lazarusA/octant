//! A colormap hovered in an overlay's picker (Settings' Layers menu)
//! previews on that overlay's colorbar in the same frame, and the base
//! layer's colorbar keeps its own.

use super::OctantApp;
use crate::app::layers::{LayerId, Source};
use egui::epaint::Mesh;
use egui::{Event, PointerButton, Pos2, RawInput, Rect, Shape, TextureId, pos2};
use std::sync::Arc;

/// Frames for areas and popups to finish fading in (half a second).
const SETTLE_FRAMES: usize = 30;
const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 1000.0));

struct Harness {
    ctx: egui::Context,
    time: f64,
}

/// One frame's meshes: colormap swatches (atlas textured) and colorbar bars
/// (vertex colored, many quads).
#[derive(Default)]
struct Painted {
    swatches: Vec<Rect>,
    bars: Vec<(Rect, Vec<egui::Color32>)>,
}

fn bounds(mesh: &Mesh) -> Rect {
    Rect::from_points(&mesh.vertices.iter().map(|v| v.pos).collect::<Vec<_>>())
}

impl Harness {
    /// Runs one frame the way the app does: the preview resets, then the
    /// floating panels and colorbars draw.
    fn frame(&mut self, app: &mut OctantApp, events: Vec<Event>) -> Painted {
        self.time += 1.0 / 60.0;
        let input = RawInput {
            screen_rect: Some(SCREEN),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| {
            app.preview_colormap = None;
            app.show_floating_panels(ui.ctx(), SCREEN, true);
        });
        output.textures_delta.clear();
        let mut painted = Painted::default();
        for clipped in &output.shapes {
            let Shape::Mesh(mesh) = &clipped.shape else {
                continue;
            };
            let mesh: &Arc<Mesh> = mesh;
            if mesh.texture_id != TextureId::default() {
                painted.swatches.push(bounds(mesh));
            } else if mesh.vertices.len() >= 256 {
                let colors = mesh.vertices.iter().map(|v| v.color).collect();
                painted.bars.push((bounds(mesh), colors));
            }
        }
        painted
    }

    fn pointer(&mut self, app: &mut OctantApp, pos: Pos2, pressed: Option<bool>) -> Painted {
        let mut events = vec![Event::PointerMoved(pos)];
        if let Some(pressed) = pressed {
            events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            });
        }
        self.frame(app, events)
    }
}

/// The colors of the bar whose panel is the colorbar of `id`.
fn bar_colors(h: &Harness, painted: &Painted, id: LayerId) -> Vec<egui::Color32> {
    let area_id = egui::Id::new(("octant_colorbar_overlay", id));
    let panel = h
        .ctx
        .memory(|m| m.area_rect(area_id))
        .expect("the colorbar is shown");
    painted
        .bars
        .iter()
        .find(|(rect, _)| panel.contains_rect(*rect))
        .map(|(_, colors)| colors.clone())
        .expect("the colorbar paints its bar")
}

#[test]
fn hovering_a_colormap_in_an_overlay_picker_previews_on_its_colorbar() {
    let mut app = OctantApp {
        show_settings_panel: true,
        show_colorbar: true,
        ..Default::default()
    };
    let overlay = app.layers.push(Source::default());
    let mut h = Harness {
        ctx: egui::Context::default(),
        time: 0.0,
    };
    // Past the areas' fade-in, so colors compare at full opacity.
    let mut painted = Painted::default();
    for _ in 0..SETTLE_FRAMES {
        painted = h.frame(&mut app, Vec::new());
    }
    let base_before = bar_colors(&h, &painted, LayerId::BASE);
    let overlay_before = bar_colors(&h, &painted, overlay);
    // The Layers menu lists the topmost overlay first.
    let layer_swatches = painted.swatches.clone();
    let swatch = layer_swatches
        .iter()
        .min_by(|a, b| a.min.y.total_cmp(&b.min.y))
        .expect("the Layers menu shows each layer's swatch")
        .center();
    h.pointer(&mut app, swatch, None);
    h.pointer(&mut app, swatch, Some(true));
    painted = h.pointer(&mut app, swatch, Some(false));
    assert_eq!(app.colormaps.target, Some(overlay));
    for _ in 0..SETTLE_FRAMES {
        painted = h.pointer(&mut app, swatch, None);
    }

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
        painted = h.pointer(&mut app, row.center(), None);
        if app.preview_colormap.is_some() && app.preview_colormap != own {
            previewed = app.preview_colormap;
            break;
        }
    }
    assert!(previewed.is_some(), "a picker row previews a colormap");

    let overlay_now = bar_colors(&h, &painted, overlay);
    assert_ne!(
        overlay_now, overlay_before,
        "the overlay's colorbar previews"
    );
    assert_eq!(bar_colors(&h, &painted, LayerId::BASE), base_before);
}
