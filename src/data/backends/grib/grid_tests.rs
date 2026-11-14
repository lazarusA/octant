//! Grid tests over edited messages: rows scanning in alternate directions, longitudes
//! wrapped across 0/360.

use super::test_files::RRFS_PEVAP;
use super::tests::{data_vars, open, read_f32, section};

#[test]
fn wrapped_longitudes_come_out_continuous() {
    use super::grid::continuous_longitudes as fix;
    // ECMWF and AIFS grids start at 180, Meteo-France's at 348: both cross 0/360.
    assert_eq!(
        fix(vec![180.0, 270.0, 0.0, 90.0]),
        [-180.0, -90.0, 0.0, 90.0]
    );
    assert_eq!(fix(vec![348.0, 359.0, 0.0, 16.0]), [-12.0, -1.0, 0.0, 16.0]);
    assert_eq!(
        fix(vec![0.0, 90.0, 180.0, 270.0]),
        [0.0, 90.0, 180.0, 270.0]
    );
    // Rows within 0..360 keep the file's values (GFS-Wave's Atlantic, 260 to 310).
    assert_eq!(fix(vec![260.0, 285.0, 310.0]), [260.0, 285.0, 310.0]);
    assert_eq!(fix(vec![0.0, 180.0, 360.0]), [0.0, 180.0, 360.0]);
}

/// NBM's grids scan adjacent rows in opposite directions (scanning mode `0x50`): the second
/// row reads reversed back. The RRFS message (`0x40`, 2 x 4) is set to alternate here.
#[test]
fn alternating_rows_read_in_one_direction() {
    let mut bytes = RRFS_PEVAP.to_vec();
    let at = section(&bytes, 3);
    assert_eq!(bytes[at + 64], 0x40);
    bytes[at + 64] = 0x50;
    let message = gribberish::message::read_message(&bytes, 0).expect("message");
    let stored: Vec<f32> = message
        .data()
        .expect("data")
        .iter()
        .map(|&v| v as f32)
        .collect();
    let reader = open(bytes.clone());
    let [var] = data_vars(&reader).try_into().expect("one variable");
    let read = read_f32(&reader, &var, &[0, 0], &[2, 4]);
    let expected: Vec<f32> = (stored[..4].iter())
        .chain(stored[4..].iter().rev())
        .copied()
        .collect();
    let same = |a: &f32, b: &f32| a == b || (a.is_nan() && b.is_nan());
    assert!(
        read.iter().zip(&expected).all(|(a, b)| same(a, b)),
        "{read:?} vs {expected:?}"
    );
}
