//! 1D coordinate arrays read whole, one chunk at a time: text as labels, numbers as `f64`,
//! with evenly spaced axes kept as start and step instead of every value.

use zarrs::array::data_type::{
    BytesDataType, FixedLengthUTF32DataType, Float16DataType, Float32DataType, StringDataType,
};
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableStorageTraits;

use super::slice::retrieve_array_subset_as_f64;
use crate::data::{CoordValues, SpacingCheck};

/// Whether `array` stores text that can label a dimension.
pub fn is_text_array<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> bool {
    let dt = array.data_type();
    dt.is::<StringDataType>() || dt.is::<FixedLengthUTF32DataType>() || dt.is::<BytesDataType>()
}

/// Reads a 1D coordinate array: text as [`CoordValues::Labels`], numbers as `Regular` or
/// `Values`. A numeric array that fails partway keeps its endpoints when they were read.
pub fn read_coordinate<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Option<CoordValues> {
    if !matches!(array.shape(), [len] if *len > 0) {
        return None;
    }
    if is_text_array(array) {
        return retrieve_array_as_strings(array).and_then(CoordValues::from_labels);
    }
    read_numbers(array)
}

/// Reads a 1D text array as trimmed labels. `None` for numeric or multi-dimensional arrays
/// and failed reads.
pub fn retrieve_array_as_strings<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Option<Vec<String>> {
    let [len] = array.shape() else {
        return None;
    };
    if !is_text_array(array) {
        return None;
    }
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

/// Reads a 1D numeric array chunk by chunk. The last chunk is decoded first, for the last
/// value the spacing check needs, and reused when the stream reaches it. A chunk that fails
/// to decode leaves the endpoints.
fn read_numbers<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Option<CoordValues> {
    let len = usize::try_from(*array.shape().first()?).ok()?;
    let chunk_count = *array.chunk_grid_shape().first()?;
    let chunk = |c: u64| {
        let subset = array.chunk_subset_bounded(&[c]).ok()?;
        retrieve_array_subset_as_f64(array, &subset).ok()
    };
    let mut head = Some(chunk(0)?);
    let mut tail = if chunk_count > 1 {
        Some(chunk(chunk_count - 1)?)
    } else {
        None
    };
    let first = *head.as_deref()?.first()?;
    let last = *tail.as_deref().or(head.as_deref())?.last()?;
    let dt = array.data_type();
    let mut check = SpacingCheck::new(first, len, last);
    check.f32_source = dt.is::<Float32DataType>() || dt.is::<Float16DataType>();

    let chunks = (0..chunk_count).map(|c| match c {
        0 => head.take(),
        c if c + 1 == chunk_count => tail.take(),
        c => chunk(c),
    });
    Some(stream(check, chunks).unwrap_or(CoordValues::Endpoints { first, last, len }))
}

/// Streams chunk values through `check`: nothing is kept while they stay evenly spaced,
/// and every value is kept from the first one that breaks the spacing (earlier ones are
/// regenerated from start and step). `None` when a chunk failed to decode.
fn stream(
    check: SpacingCheck,
    chunks: impl Iterator<Item = Option<Vec<f64>>>,
) -> Option<CoordValues> {
    let mut uneven: Option<Vec<f64>> = None;
    let mut index = 0usize;
    for values in chunks {
        for v in values? {
            match uneven.as_mut() {
                Some(out) => out.push(v),
                None if check.fits(index, v) => {}
                None => {
                    let mut out = Vec::with_capacity(check.len);
                    out.extend((0..index).map(|i| check.expected(i)));
                    out.push(v);
                    uneven = Some(out);
                }
            }
            index += 1;
        }
    }
    Some(match uneven {
        Some(values) => CoordValues::Values(values.into()),
        None => check.regular(),
    })
}

/// Strips the NUL padding of fixed-width strings and surrounding whitespace.
fn clean_label(label: &str) -> String {
    label
        .trim_matches(|c: char| c == '\0' || c.is_whitespace())
        .to_string()
}
