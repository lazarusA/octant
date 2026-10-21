//! Predictors on truncated input: the partial last sample stays as it is.

use async_tiff::reader::Endianness;
use async_tiff::tags::Predictor;

use super::predictors::unpredict_buffer;

/// Two 16-bit samples and one stray byte, as a corrupt tile delivers them.
#[test]
fn horizontal_predictor_on_a_truncated_row_does_not_panic() {
    let row = vec![1, 0, 2, 0, 3];
    let out = unpredict_buffer(
        row,
        Predictor::Horizontal,
        1,
        16,
        3,
        Endianness::LittleEndian,
    )
    .expect("a truncated row decodes");
    assert_eq!(out, [1, 0, 3, 0, 3]);
}
