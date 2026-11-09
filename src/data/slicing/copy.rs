//! Buffer copying routines for strided and contiguous hyperslab slices.

/// Copies values for contiguous row and column slices.
pub fn copy_contiguous_slice(
    values: &[f32],
    base_offset: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    width: usize,
) -> Vec<f32> {
    let slice_len = width * (y_end - y_start);
    let slice_start = base_offset + y_start * stride_y;
    let slice_end = slice_start + slice_len;

    if slice_end <= values.len() {
        values[slice_start..slice_end].to_vec()
    } else {
        let mut result = Vec::with_capacity(slice_len);
        for y in y_start..y_end {
            let row_start = base_offset + y * stride_y;
            let row_end = row_start + width;
            if row_end <= values.len() {
                result.extend_from_slice(&values[row_start..row_end]);
            } else {
                for x in 0..width {
                    result.push(values.get(row_start + x).copied().unwrap_or(f32::NAN));
                }
            }
        }
        result
    }
}

/// Copies values when rows are contiguous in X (stride_x == 1).
pub fn copy_row_contiguous_slice(
    values: &[f32],
    base_offset: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    x_start: usize,
    width: usize,
) -> Vec<f32> {
    let mut result = Vec::with_capacity(width * (y_end - y_start));
    let rows = (y_start, y_end, stride_y);
    copy_row_contiguous_into(&mut result, values, base_offset, rows, x_start, width);
    result
}

/// Appends the rows `y_start..y_end` (`rows`: start, end, stride) of `width`
/// contiguous values from `x_start` at `base_offset` to `result`; values past
/// the end read NaN.
pub fn copy_row_contiguous_into(
    result: &mut Vec<f32>,
    values: &[f32],
    base_offset: usize,
    (y_start, y_end, stride_y): (usize, usize, usize),
    x_start: usize,
    width: usize,
) {
    for y in y_start..y_end {
        let row_start = base_offset + y * stride_y + x_start;
        let row_end = row_start + width;
        if row_end <= values.len() {
            result.extend_from_slice(&values[row_start..row_end]);
        } else {
            for x in 0..width {
                result.push(values.get(row_start + x).copied().unwrap_or(f32::NAN));
            }
        }
    }
}

/// Copies values for arbitrary strided X and Y slices.
#[allow(clippy::too_many_arguments)]
pub fn copy_strided_slice(
    values: &[f32],
    base_offset: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    x_start: usize,
    x_end: usize,
    stride_x: usize,
) -> Vec<f32> {
    let width = x_end.saturating_sub(x_start);
    let height = y_end.saturating_sub(y_start);
    let mut result = Vec::with_capacity(width * height);
    let rows = (y_start, y_end, stride_y);
    copy_strided_into(
        &mut result,
        values,
        base_offset,
        rows,
        (x_start, x_end, stride_x),
    );
    result
}

/// Appends the strided values of rows and columns (each start, end, stride)
/// at `base_offset` to `result`; values past the end read NaN.
pub fn copy_strided_into(
    result: &mut Vec<f32>,
    values: &[f32],
    base_offset: usize,
    (y_start, y_end, stride_y): (usize, usize, usize),
    (x_start, x_end, stride_x): (usize, usize, usize),
) {
    for y in y_start..y_end {
        let row_start = base_offset + y * stride_y;
        for x in x_start..x_end {
            let idx = row_start + x * stride_x;
            result.push(values.get(idx).copied().unwrap_or(f32::NAN));
        }
    }
}
