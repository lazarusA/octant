//! Minor ticks: round subdivisions, cell centers, and their agreement with
//! the cell the hover picks.

use std::sync::Arc;

use super::minor::{CellTicks, Lattice, subdivisions};
use super::steps::{AxisSpec, axis_ticks};
use super::style::MinorPlacement;
use crate::data::CoordinateGrid;

fn spec<'a>(start: f64, end: f64, px: f32, cells: CellTicks<'a>) -> AxisSpec<'a> {
    AxisSpec {
        start,
        end,
        title: "x",
        units: None,
        px,
        cells,
        minor: Some(MinorPlacement::Cells),
        majors: true,
    }
}

/// The minor ticks' values, sorted.
fn minor_values(spec: &AxisSpec<'_>) -> Vec<f64> {
    let ticks = axis_ticks(spec);
    let span = spec.end - spec.start;
    let mut v: Vec<f64> = ticks
        .minor()
        .iter()
        .map(|&f| spec.start + f64::from(f) * span)
        .map(|v| (v * 1e4).round() / 1e4)
        .collect();
    v.sort_by(f64::total_cmp);
    v
}

fn round(start: f64, end: f64, px: f32) -> AxisSpec<'static> {
    AxisSpec {
        minor: Some(MinorPlacement::Round),
        ..spec(start, end, px, CellTicks::None)
    }
}

#[test]
fn steps_split_into_whole_parts() {
    for (step, n) in [
        (1.0, 5),
        (2.0, 4),
        (5.0, 5),
        (0.2, 4),
        (500.0, 5),
        (0.01, 5),
    ] {
        assert_eq!(subdivisions(step, false), n, "step {step}");
    }
    let degrees = [(10.0, 5), (15.0, 3), (30.0, 3), (45.0, 3), (60.0, 2)];
    for (step, n) in degrees.into_iter().chain([(90.0, 3), (180.0, 6)]) {
        assert_eq!(subdivisions(step, true), n, "{step}°");
    }
}

#[test]
fn round_minors_fill_the_axis_but_never_a_major() {
    // Majors every 2 (42..48), minors every 0.5, out to both ends.
    let v = minor_values(&round(40.3, 49.7, 1000.0));
    assert_eq!(v.first(), Some(&40.5));
    assert_eq!(v.last(), Some(&49.5));
    assert!(v.iter().all(|x| x % 2.0 != 0.0));
    assert_eq!(v.len(), 19 - 4);
}

#[test]
fn round_minors_take_the_majors_place_when_majors_are_off() {
    let off = AxisSpec {
        majors: false,
        ..round(40.3, 49.7, 1000.0)
    };
    assert_eq!(minor_values(&off).len(), 19);
}

#[test]
fn crowded_minors_are_dropped() {
    // 0.5 over 9.4 units on 50 px: under 3 px apart.
    assert!(minor_values(&round(40.3, 49.7, 50.0)).is_empty());
}

#[test]
fn time_axes_have_no_round_minors() {
    let time = AxisSpec {
        title: "time",
        units: Some("days since 2000-01-01"),
        ..round(0.0, 30.0, 1000.0)
    };
    assert!(axis_ticks(&time).minor().is_empty());
}

#[test]
fn no_minors_unless_asked() {
    let none = AxisSpec {
        minor: None,
        ..round(0.0, 10.0, 1000.0)
    };
    assert!(axis_ticks(&none).minor().is_empty());
}

fn even(plotted: Option<Lattice>, detail: Option<Lattice>) -> CellTicks<'static> {
    plotted.map_or(CellTicks::None, |plotted| CellTicks::Even {
        plotted,
        detail,
    })
}

#[test]
fn heatmap_cells_tick_at_their_centers() {
    let cells = even(Lattice::cells(0.0, 10.0, 10), None);
    let expected: Vec<f64> = (0..10).map(|i| i as f64 + 0.5).collect();
    assert_eq!(minor_values(&spec(0.0, 10.0, 1000.0, cells)), expected);
    // Only the visible ones, on a descending (y_down) axis too.
    assert_eq!(
        minor_values(&spec(7.0, 3.0, 1000.0, cells)),
        [3.5, 4.5, 5.5, 6.5]
    );
}

#[test]
fn line_samples_tick_except_under_majors() {
    let cells = even(Lattice::samples(0.0, 10.0, 11), None);
    // Majors every 2 cover the even samples.
    assert_eq!(
        minor_values(&spec(0.0, 10.0, 1000.0, cells)),
        [1.0, 3.0, 5.0, 7.0, 9.0]
    );
}

#[test]
fn narrow_cells_fall_back_to_round_minors() {
    let cells = even(Lattice::cells(0.0, 10.0, 1000), None);
    let cell_ticks = minor_values(&spec(0.0, 10.0, 1000.0, cells));
    assert_eq!(cell_ticks, minor_values(&round(0.0, 10.0, 1000.0)));
    assert!(!cell_ticks.is_empty());
}

#[test]
fn a_detail_tile_replaces_the_coarse_cells_it_covers() {
    let cells = even(Lattice::cells(0.0, 10.0, 10), Lattice::cells(2.0, 4.0, 8));
    let v = minor_values(&spec(0.0, 10.0, 1000.0, cells));
    let fine: Vec<f64> = (0..8).map(|i| 2.125 + 0.25 * i as f64).collect();
    let coarse = [0.5, 1.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5];
    let mut expected: Vec<f64> = fine.into_iter().chain(coarse).collect();
    expected.sort_by(f64::total_cmp);
    assert_eq!(v, expected);
}

#[test]
fn irregular_cells_tick_at_their_values_in_view() {
    let ascending = [-60.0, -45.0, -21.0, 5.0, 33.0, 61.0];
    let descending = [61.0, 33.0, 5.0, -21.0, -45.0, -60.0];
    for values in [&ascending[..], &descending[..]] {
        let s = spec(-50.0, 50.0, 1000.0, CellTicks::Values(values));
        assert_eq!(minor_values(&s), [-45.0, -21.0, 5.0, 33.0]);
    }
}

#[test]
fn irregular_cells_on_a_major_are_left_to_it() {
    let values = [-20.0, -7.0, 13.0];
    let s = spec(-50.0, 50.0, 1000.0, CellTicks::Values(&values));
    assert_eq!(minor_values(&s), [-7.0, 13.0]);
}

#[test]
fn crowded_irregular_cells_fall_back_to_round_minors() {
    let values = [0.0, 1.0, 2.0, 2.01, 3.0];
    let s = spec(0.0, 3.0, 300.0, CellTicks::Values(&values));
    assert_eq!(minor_values(&s), minor_values(&round(0.0, 3.0, 300.0)));
}

#[test]
fn the_hover_picks_the_cell_of_each_regular_cell_tick() {
    let (lo, hi, w) = (-180.0, 180.0, 36);
    let cells = even(Lattice::cells(lo, hi, w), None);
    for v in minor_values(&spec(lo, hi, 1000.0, cells)) {
        let norm = ((v - lo) / (hi - lo)) as f32;
        let (px, _) = CoordinateGrid::GlobalRegular.find_cell_from_norm(norm, 0.5, w, 18);
        assert_eq!(lo + (px as f64 + 0.5) * 10.0, v);
    }
}

#[test]
fn the_hover_picks_the_cell_of_each_irregular_cell_tick() {
    // Rows north to south: norm_y 0 at the top, the axis's largest value.
    let lat: Arc<[f32]> = Arc::from([80.0, 52.0, 30.0, 11.0, -9.0, -40.0, -75.0]);
    let grid = CoordinateGrid::Irregular1D {
        coords_x: Arc::from([0.0, 1.0]),
        coords_y: lat.clone(),
        lon_bounds: (0.0, 1.0),
        lat_bounds: (-75.0, 80.0),
    };
    // The y axis from its bottom (fract 0) to its top.
    let s = AxisSpec {
        majors: false,
        ..spec(-75.0, 80.0, 1000.0, CellTicks::Values(&lat))
    };
    let ticks = axis_ticks(&s);
    assert_eq!(ticks.minor().len(), lat.len());
    for &f in ticks.minor() {
        let value = -75.0 + f64::from(f) * 155.0;
        let (_, py) = grid.find_cell_from_norm(0.5, 1.0 - f, 2, lat.len());
        assert!(
            (f64::from(lat[py]) - value).abs() < 1e-3,
            "{value} -> {}",
            lat[py]
        );
    }
}
