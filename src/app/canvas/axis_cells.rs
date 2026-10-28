//! The data cells along a plot's axes, where cell ticks go: the cells the
//! hover snaps to.

use crate::app::OctantApp;
use crate::data::{CoordinateGrid, MatrixData};
use crate::ui::axes::{CellTicks, Lattice};

/// A heatmap's cells along x and y over the domains the axes show: a
/// regular grid's, with a shown detail tile's finer ones over the part of the
/// quad it covers, or an irregular grid's coordinate values.
pub(super) fn heatmap_cells<'a>(
    app: &OctantApp,
    matrix: &'a MatrixData,
    (x_dom, y_dom): ((f64, f64), (f64, f64)),
    y_down: bool,
) -> (CellTicks<'a>, CellTicks<'a>) {
    match &matrix.grid {
        CoordinateGrid::GlobalRegular | CoordinateGrid::RegionalRegular { .. } => {
            let (w, h) = app.active_data_dimensions_2d();
            let (tile_x, tile_y) = detail_cells(app, (x_dom, y_dom), y_down);
            (
                even(Lattice::cells(x_dom.0, x_dom.1, w), tile_x),
                even(Lattice::cells(y_dom.0, y_dom.1, h), tile_y),
            )
        }
        CoordinateGrid::Irregular1D {
            coords_x, coords_y, ..
        } => (values(coords_x, x_dom), values(coords_y, y_dom)),
        _ => (CellTicks::None, CellTicks::None),
    }
}

/// A line plot's samples along x, `count` of them from the first to the last.
pub(super) fn line_cells(x_dom: (f64, f64), count: usize) -> CellTicks<'static> {
    even(Lattice::samples(x_dom.0, x_dom.1, count), None)
}

fn even(plotted: Option<Lattice>, detail: Option<Lattice>) -> CellTicks<'static> {
    plotted.map_or(CellTicks::None, |plotted| CellTicks::Even {
        plotted,
        detail,
    })
}

/// The cells of the base layer's shown detail tile along x and y, placed
/// where it is drawn on the quad (`DetailTile::uv`, `v` from the top).
fn detail_cells(
    app: &OctantApp,
    (x_dom, y_dom): ((f64, f64), (f64, f64)),
    y_down: bool,
) -> (Option<Lattice>, Option<Lattice>) {
    let detail = &app.layers.base.data.detail;
    let Some((tile, m)) = detail
        .tile
        .as_ref()
        .and_then(|t| Some((t, t.matrix.as_ref()?)))
    else {
        return (None, None);
    };
    let [u0, v0, u1, v1] = tile.uv;
    let at = |dom: (f64, f64), t: f32| dom.0.min(dom.1) + f64::from(t) * (dom.1 - dom.0).abs();
    // y grows up the quad unless the image's rows run down.
    let up = |v: f32| if y_down { v } else { 1.0 - v };
    (
        Lattice::cells(at(x_dom, u0), at(x_dom, u1), m.width),
        Lattice::cells(at(y_dom, up(v0)), at(y_dom, up(v1)), m.height),
    )
}

/// An irregular axis's cell centers, when they run from one end of the
/// axis's domain to the other (the axis then places them by value).
fn values(coords: &[f32], dom: (f64, f64)) -> CellTicks<'_> {
    let (Some(&a), Some(&b)) = (coords.first(), coords.last()) else {
        return CellTicks::None;
    };
    let tolerance = (dom.1 - dom.0).abs() * 1e-4;
    let near = |c: f32, d: f64| (f64::from(c) - d).abs() <= tolerance;
    if (near(a, dom.0) && near(b, dom.1)) || (near(a, dom.1) && near(b, dom.0)) {
        CellTicks::Values(coords)
    } else {
        CellTicks::None
    }
}
