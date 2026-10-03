//! "OCTANT" wordmark built from little cubes on a 5x5 grid per letter.
//!
//! Each lit cell is a block: a front square plus right and bottom side faces
//! shaded like the hero cube. Below 7 physical pixels per cell the side faces
//! collapse into a flat drop shadow.

use super::face_colors;
use egui::{Color32, Mesh, Pos2, Rect, Response, Sense, Shape, Ui, Vec2, Widget, pos2, vec2};

/// Letter bitmaps, one `u8` per row, bit 4 = leftmost column.
const O: [u8; 5] = [0b01110, 0b10001, 0b10001, 0b10001, 0b01110];
const C: [u8; 5] = [0b01111, 0b10000, 0b10000, 0b10000, 0b01111];
const T: [u8; 5] = [0b11111, 0b00100, 0b00100, 0b00100, 0b00100];
const A: [u8; 5] = [0b01110, 0b10001, 0b11111, 0b10001, 0b10001];
const N: [u8; 5] = [0b10001, 0b11001, 0b10101, 0b10011, 0b10001];
pub(super) const WORD: [[u8; 5]; 6] = [O, C, T, A, N, T];

/// Grid size: six 5-column letters with one empty column between them.
pub(super) const COLS: usize = WORD.len() * 6 - 1;
pub(super) const ROWS: usize = 5;
/// Cells smaller than this many physical pixels drop to the flat style.
const BLOCK_MIN_PX: f32 = 7.0;

/// Lit cells as (row, column within the whole word).
fn cells() -> impl Iterator<Item = (usize, usize)> {
    WORD.iter().enumerate().flat_map(|(li, glyph)| {
        (0..ROWS).flat_map(move |r| {
            (0..5)
                .filter(move |&c| (glyph[r] >> (4 - c)) & 1 == 1)
                .map(move |c| (r, li * 6 + c))
        })
    })
}

/// Pixel-snapped metrics, in points but derived from whole physical pixels.
#[derive(Clone, Copy, Debug)]
pub(super) struct Metrics {
    pub cell: f32,
    pub side: f32,
    pub depth: f32,
    pub blocks: bool,
}

impl Metrics {
    pub fn new(cell_pt: f32, ppp: f32) -> Self {
        let cell_px = (cell_pt * ppp).round().max(3.0);
        let depth_px = (cell_px * 0.3).round().max(1.0);
        Self {
            cell: cell_px / ppp,
            side: (cell_px - 1.0) / ppp,
            depth: depth_px / ppp,
            blocks: cell_px >= BLOCK_MIN_PX,
        }
    }

    /// Outer size of the wordmark, including the side faces.
    pub fn size(&self) -> Vec2 {
        vec2(
            COLS as f32 * self.cell + self.depth,
            ROWS as f32 * self.cell + self.depth,
        )
    }
}

/// The block wordmark widget.
#[derive(Clone, Copy, Debug)]
pub struct Wordmark {
    cell: f32,
}

impl Wordmark {
    /// Wordmark with cells of roughly `cell` points (snapped to whole pixels).
    pub fn new(cell: f32) -> Self {
        Self { cell }
    }
}

impl Widget for Wordmark {
    fn ui(self, ui: &mut Ui) -> Response {
        let ppp = ui.ctx().pixels_per_point();
        let m = Metrics::new(self.cell, ppp);
        let (rect, response) = ui.allocate_exact_size(m.size(), Sense::hover());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, "Octant"));
        if ui.is_rect_visible(rect) {
            let origin = pos2(
                (rect.min.x * ppp).round() / ppp,
                (rect.min.y * ppp).round() / ppp,
            );
            paint(ui, origin, m);
        }
        response
    }
}

/// Top-left corner of word cell (`row`, `col`).
fn cell_pos(origin: Pos2, m: Metrics, row: usize, col: usize) -> Pos2 {
    origin + vec2(col as f32 * m.cell, row as f32 * m.cell)
}

fn paint(ui: &Ui, origin: Pos2, m: Metrics) {
    let visuals = ui.visuals();
    let base = face_colors(visuals, visuals.strong_text_color());
    let painter = ui.painter();

    // Side faces (or flat shadows) first, then every front on top.
    let mut sides = Mesh::default();
    for (r, col) in cells() {
        let min = cell_pos(origin, m, r, col);
        add_sides(&mut sides, min, m.side, m.depth, base, m.blocks);
    }
    painter.add(Shape::mesh(sides));
    for (r, col) in cells() {
        let front = Rect::from_min_size(cell_pos(origin, m, r, col), Vec2::splat(m.side));
        painter.rect_filled(front, 0.0, base[0]);
    }
}

/// Append the right and bottom faces of a block at `min` (or its flat drop
/// shadow when `blocks` is false) to `mesh`.
fn add_sides(mesh: &mut Mesh, min: Pos2, s: f32, o: f32, faces: [Color32; 3], blocks: bool) {
    let (x, y) = (min.x, min.y);
    if !blocks {
        let shadow = [
            pos2(x + o, y + o),
            pos2(x + o + s, y + o),
            pos2(x + o + s, y + o + s),
            pos2(x + o, y + o + s),
        ];
        quad(mesh, shadow, faces[2]);
        return;
    }
    let right = [
        pos2(x + s, y),
        pos2(x + s + o, y + o),
        pos2(x + s + o, y + s + o),
        pos2(x + s, y + s),
    ];
    let bottom = [
        pos2(x, y + s),
        pos2(x + s, y + s),
        pos2(x + s + o, y + s + o),
        pos2(x + o, y + s + o),
    ];
    quad(mesh, right, faces[1]);
    quad(mesh, bottom, faces[2]);
}

fn quad(mesh: &mut Mesh, pts: [Pos2; 4], color: Color32) {
    let i = mesh.vertices.len() as u32;
    for p in pts {
        mesh.colored_vertex(p, color);
    }
    mesh.add_triangle(i, i + 1, i + 2);
    mesh.add_triangle(i, i + 2, i + 3);
}
