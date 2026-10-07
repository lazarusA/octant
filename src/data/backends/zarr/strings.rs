//! Text coordinate arrays (`string`, `fixed_length_utf32`, variable-length `bytes`) read as
//! one label per index.

use std::ops::Range;

use zarrs::array::data_type::{BytesDataType, FixedLengthUTF32DataType, StringDataType};
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableStorageTraits;

/// Most labels read from one text coordinate; longer text arrays are left unread.
pub const MAX_LABELS: usize = 4096;

/// Whether `array` stores text that can label a dimension.
pub fn is_text_array<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> bool {
    let dt = array.data_type();
    dt.is::<StringDataType>() || dt.is::<FixedLengthUTF32DataType>() || dt.is::<BytesDataType>()
}

/// Whether `array` is a 1D text coordinate short enough to read every label of.
pub fn is_label_array<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> bool {
    matches!(array.shape(), [len] if (1..=MAX_LABELS as u64).contains(len)) && is_text_array(array)
}

/// The index ranges a coordinate preload fetches for a 1D coordinate of `count` values:
/// all of a label coordinate (`array`, when it could be opened), else the first and last.
#[allow(clippy::single_range_in_vec_init)]
pub fn coordinate_preload_ranges<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: Option<&Array<TStorage>>,
    count: u64,
) -> Vec<Range<u64>> {
    match count {
        0 => Vec::new(),
        _ if array.is_some_and(is_label_array) => vec![0..count],
        1 => vec![0..1],
        _ => vec![0..1, count - 1..count],
    }
}

/// Reads a 1D text array as trimmed labels. `None` for numeric or multi-dimensional arrays,
/// arrays longer than [`MAX_LABELS`], and failed reads.
pub fn retrieve_array_as_strings<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Option<Vec<String>> {
    if !is_label_array(array) {
        return None;
    }
    let len = array.shape().first()?;
    #[allow(clippy::single_range_in_vec_init)]
    let subset = ArraySubset::new_with_ranges(&[0..*len]);
    let dt = array.data_type();
    let labels: Vec<String> = if dt.is::<StringDataType>() {
        array.retrieve_array_subset::<Vec<String>>(&subset).ok()?
    } else if dt.is::<FixedLengthUTF32DataType>() {
        let chars = array
            .retrieve_array_subset::<Vec<Vec<char>>>(&subset)
            .ok()?;
        chars.into_iter().map(|c| c.into_iter().collect()).collect()
    } else {
        let bytes = array.retrieve_array_subset::<Vec<Vec<u8>>>(&subset).ok()?;
        bytes
            .iter()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .collect()
    };
    Some(labels.iter().map(|l| clean_label(l)).collect())
}

/// Strips the NUL padding of fixed-width strings and surrounding whitespace.
fn clean_label(label: &str) -> String {
    label
        .trim_matches(|c: char| c == '\0' || c.is_whitespace())
        .to_string()
}
