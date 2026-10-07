//! 1D coordinate arrays read whole: text as labels in one read, numbers as `f64` in spans
//! of chunks, with evenly spaced axes kept as start and step instead of every value.

use zarrs::array::data_type::{
    BFloat16DataType, BytesDataType, FixedLengthUTF32DataType, Float16DataType, Float32DataType,
    StringDataType,
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
            .into_iter()
            .map(|b| {
                String::from_utf8(b)
                    .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
            })
            .collect()
    };
    Some(labels.into_iter().map(clean_label).collect())
}

/// Chunks read per subset after the first and last: zarrs decodes the chunks of one
/// subset concurrently, so long coordinates of small chunks read in parallel.
const COORD_CHUNK_SPAN: u64 = 64;

/// Reads a 1D numeric array in spans of chunks. The first and last chunk are decoded first,
/// for the endpoints the spacing check needs, and reused when the stream reaches them. A
/// span that fails to decode leaves the endpoints.
fn read_numbers<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
) -> Option<CoordValues> {
    let len = usize::try_from(*array.shape().first()?).ok()?;
    let chunk_count = *array.chunk_grid_shape().first()?;
    // Chunks `first..=last` as one subset.
    let span = |first: u64, last: u64| {
        let start = array.chunk_subset_bounded(&[first]).ok()?.to_ranges();
        let end = array.chunk_subset_bounded(&[last]).ok()?.to_ranges();
        #[allow(clippy::single_range_in_vec_init)]
        let subset = ArraySubset::new_with_ranges(&[start.first()?.start..end.first()?.end]);
        retrieve_array_subset_as_f64(array, &subset).ok()
    };
    let last_chunk = chunk_count.checked_sub(1)?;
    let head = span(0, 0)?;
    let tail = if last_chunk > 0 {
        Some(span(last_chunk, last_chunk)?)
    } else {
        None
    };
    let first = *head.first()?;
    let last = *tail.as_deref().unwrap_or(&head).last()?;
    let dt = array.data_type();
    let mut check = SpacingCheck::new(first, len, last);
    check.f32_source =
        dt.is::<Float32DataType>() || dt.is::<Float16DataType>() || dt.is::<BFloat16DataType>();

    let middle = (1..last_chunk)
        .step_by(COORD_CHUNK_SPAN as usize)
        .map(|c| span(c, (c + COORD_CHUNK_SPAN - 1).min(last_chunk - 1)));
    let chunks = std::iter::once(Some(head))
        .chain(middle)
        .chain(tail.map(Some));
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

/// Strips the NUL padding of fixed-width strings and surrounding whitespace, reusing the
/// label's buffer when there is nothing to strip.
fn clean_label(label: String) -> String {
    let trimmed = label.trim_matches(|c: char| c == '\0' || c.is_whitespace());
    if trimmed.len() == label.len() {
        label
    } else {
        trimmed.to_string()
    }
}
