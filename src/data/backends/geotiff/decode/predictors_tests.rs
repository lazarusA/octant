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

/// TIFF floating-point predictor over one row of `values`: MSB-first byte
/// planes, then byte differences (independent of the file's byte order).
fn float_predicted(values: &[f32]) -> Vec<u8> {
    let n = values.len();
    let mut planes = vec![0u8; n * 4];
    for (i, v) in values.iter().enumerate() {
        for (b, byte) in v.to_be_bytes().iter().enumerate() {
            planes[b * n + i] = *byte;
        }
    }
    for i in (1..planes.len()).rev() {
        planes[i] = planes[i].wrapping_sub(planes[i - 1]);
    }
    planes
}

/// The floating-point predictor decodes alike in big- and little-endian files.
#[test]
fn float_predictor_ignores_the_file_byte_order() {
    let values = [1.5f32, -2.25, 1000.0, 0.125];
    let expected: Vec<u8> = values.iter().flat_map(|v| v.to_ne_bytes()).collect();
    for endianness in [Endianness::LittleEndian, Endianness::BigEndian] {
        let encoded = float_predicted(&values);
        let out = unpredict_buffer(encoded, Predictor::FloatingPoint, 1, 32, 4, endianness)
            .expect("decodes");
        assert_eq!(out, expected, "{endianness:?}");
    }
}
