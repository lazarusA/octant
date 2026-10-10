//! Line layouts (rows, columns, a picked line) and the payload key.

use super::line_profile::LineLayout;
use crate::app::OctantApp;
use crate::data::{MatrixData, VolumeData};

const VALUES: [f32; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

fn app_with_matrix() -> OctantApp {
    let mut app = OctantApp::default();
    let matrix = MatrixData::new(3, 2, vec![1.0; 6], 1.0, 1.0, "rows".to_string(), 1);
    app.layers.base.data.matrix = Some(matrix);
    app
}

#[test]
fn the_payload_key_follows_the_data_and_its_layout() {
    let mut app = app_with_matrix();
    let key = app.line_payload_key();
    assert_eq!(app.line_payload_key(), key, "nothing changed");
    app.layers.base.data.touch_matrix();
    let touched = app.line_payload_key();
    assert_ne!(touched, key, "the data changed");
    app.line_profile_slice_idx = 1;
    let last = app.line_payload_key();
    assert_ne!(last, touched, "another line picked");
    app.line_profile_slice_idx = 5;
    assert_eq!(
        app.line_payload_key(),
        last,
        "past the last line: the same line"
    );
    app.line_plot_all_series = true;
    let all = app.line_payload_key();
    app.line_profile_slice_idx = 0;
    assert_eq!(app.line_payload_key(), all, "every line drawn: no pick");
    app.line_profile_dim_idx = 1;
    assert_ne!(app.line_payload_key(), all, "lines along another axis");
}

#[test]
fn volume_changes_leave_matrix_lines_alone() {
    let mut app = app_with_matrix();
    let volume = VolumeData::new(1, 1, 2, vec![1.0, 2.0], 1.0, 2.0, "z".to_string());
    app.layers.base.data.volume = Some(volume);
    let rows = app.line_payload_key();
    app.layers.base.data.touch_volume();
    assert_eq!(
        app.line_payload_key(),
        rows,
        "lines along X read the matrix"
    );
    app.line_profile_dim_idx = 2;
    let rays = app.line_payload_key();
    app.layers.base.data.touch_volume();
    assert_ne!(app.line_payload_key(), rays, "rays along Z read the volume");
}

#[test]
fn rows_and_columns_of_a_matrix() {
    // 3 wide, 2 high.
    let rows = LineLayout::lines(3, 2, (3, 1), None);
    assert_eq!((rows.line_count, rows.pick), (2, None));
    assert_eq!(rows.value(&VALUES, 1, 2), 6.0);
    let cols = LineLayout::lines(2, 3, (1, 3), None);
    assert_eq!(cols.value(&VALUES, 2, 1), 6.0);
    assert_eq!(cols.value(&VALUES, 0, 1), 4.0);
}

#[test]
fn rows_and_columns_read_in_order_with_nan_past_the_end() {
    let rows = LineLayout::lines(3, 2, (3, 1), None);
    assert_eq!(rows.row(&VALUES, 1).collect::<Vec<_>>(), [4.0, 5.0, 6.0]);
    assert_eq!(rows.column(&VALUES, 2).collect::<Vec<_>>(), [3.0, 6.0]);
    let short: Vec<f32> = rows.row(&VALUES[..5], 1).collect();
    assert_eq!(short[..2], [4.0, 5.0]);
    assert!(short[2].is_nan());
    assert!(rows.has_data(&VALUES, 1));
    assert!(!rows.has_data(&[f32::NAN; 6], 1));
    assert_eq!(rows.payload_bytes(2), Some(2 * 4 * 4));
}

#[test]
fn a_picked_line_is_clamped_and_short_data_reads_nan() {
    let row = LineLayout::lines(3, 2, (3, 1), Some(9));
    assert_eq!((row.line_count, row.pick), (1, Some((1, 2))));
    assert_eq!(row.value(&VALUES, 0, 0), 4.0);
    assert!(row.value(&VALUES[..4], 0, 2).is_nan());
    let none = LineLayout::lines(3, 0, (3, 1), Some(0));
    assert_eq!((none.line_count, none.pick), (0, None));
}
