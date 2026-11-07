//! The gizmo drawn in the crop tool's style (`crop_overlay`): edges as thin
//! accent lines with the crop tool's pill at their middle, box corners as L
//! brackets along the edges meeting there (`strokes`). Guides on faces
//! turned away are dimmed and their edges dotted. The hovered or held grip
//! keeps the accent's hue, brighter and thicker over a halo of the canvas
//! color, which parts it from any colormap underneath.

use super::guide::Guide;
use super::space::Space;
use super::strokes::{corner, line, pill};
use super::{Grip, handles, planes, shown_space, state};
use crate::app::{OctantApp, SliceMode, Slicer};
use crate::ui::crop_overlay::pill_outline;
use crate::ui::icons::IconTone;
use egui::{Color32, Painter};

/// Alpha factor of guides on faces turned away.
const HIDDEN: f32 = 0.4;

/// The gizmo's colors: the accent and the pill outline (as the crop
/// tool's), dimmed for hidden guides; for the hovered or held grip a
/// brighter accent and its halo.
#[derive(Clone, Copy)]
pub(super) struct Colors {
    pub accent: Color32,
    pub outline: Color32,
    hover: Color32,
    pub halo: Color32,
}

impl Colors {
    fn of(visuals: &egui::Visuals) -> Self {
        let accent = IconTone::Accent.color(visuals);
        let hover = if visuals.dark_mode {
            accent.lerp_to_gamma(Color32::WHITE, 0.3)
        } else {
            accent.lerp_to_gamma(Color32::BLACK, 0.25)
        };
        Self {
            accent,
            outline: pill_outline(visuals),
            hover,
            halo: visuals.panel_fill.gamma_multiply(0.5),
        }
    }

    /// The colors of a guide, shown or on faces turned away, lit or not.
    pub fn of_guide(self, shown: bool, lit: bool) -> Self {
        let accent = if lit { self.hover } else { self.accent };
        if shown {
            Self { accent, ..self }
        } else {
            Self {
                accent: accent.gamma_multiply(HIDDEN),
                outline: self.outline.gamma_multiply(HIDDEN),
                ..self
            }
        }
    }
}

/// Draws the gizmo over the plot; not during export.
pub fn draw(app: &OctantApp, ui: &egui::Ui, response: &egui::Response) {
    let Some(space) = shown_space(app, response.rect) else {
        return;
    };
    let slice = &app.plot_configs.slice;
    let hovered = state(ui, response).hovered;
    let painter = ui.painter().with_clip_rect(response.rect);
    let c = Colors::of(ui.visuals());
    if slice.mode == SliceMode::Planes {
        let (edges, n) = planes::edges(&slice.planes);
        let grips = edges[..n].iter().map(|e| (*e, Grip::Plane(*e)));
        draw_guides(&painter, &space, slice, grips, hovered, c);
    } else {
        let (all, n) = handles::all(space.dims);
        let grips = all[..n].iter().map(|h| (*h, Grip::Box(*h)));
        draw_guides(&painter, &space, slice, grips, hovered, c);
    }
}

/// Edges' lines, then their pills, then corners over them.
fn draw_guides<G: Guide>(
    p: &Painter,
    space: &Space,
    slice: &Slicer,
    guides: impl Iterator<Item = (G, Grip)> + Clone,
    hovered: Option<Grip>,
    c: Colors,
) {
    let edges = guides.clone().filter(|(g, _)| !g.is_corner(space.dims));
    for (g, grip) in edges.clone() {
        let shown = |t| g.visible(space, slice, t);
        line(p, &g.polyline(space, slice), shown, grip.lit_by(hovered), c);
    }
    for (g, grip) in edges {
        let at = |t| space.screen(g.point(slice, t));
        let lit = grip.lit_by(hovered);
        pill(p, at, c.of_guide(g.visible(space, slice, 0.5), lit), lit);
    }
    for (_, grip) in guides {
        if let Grip::Box(h) = grip
            && h.is_corner(space.dims)
        {
            corner(p, space, slice, &h, grip.lit_by(hovered), c);
        }
    }
}
