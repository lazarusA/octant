//! Helper for retrieving Zarr array subsets converted to f32.

use std::error::Error;
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableWritableListableStorageTraits;

/// Dtype-conversion helper for reading array subsets as f32.
pub fn retrieve_array_subset_as_f32(
    array: &Array<dyn ReadableWritableListableStorageTraits>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error>> {
    let dt_str = array.data_type().to_string().to_lowercase();
    if dt_str.contains("float64") || dt_str.contains("f64") {
        let vals = array.retrieve_array_subset::<Vec<f64>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("int64") || dt_str.contains("i64") {
        let vals = array.retrieve_array_subset::<Vec<i64>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("int32") || dt_str.contains("i32") {
        let vals = array.retrieve_array_subset::<Vec<i32>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("uint64") || dt_str.contains("u64") {
        let vals = array.retrieve_array_subset::<Vec<u64>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("uint32") || dt_str.contains("u32") {
        let vals = array.retrieve_array_subset::<Vec<u32>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("int16") || dt_str.contains("i16") {
        let vals = array.retrieve_array_subset::<Vec<i16>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("uint16") || dt_str.contains("u16") {
        let vals = array.retrieve_array_subset::<Vec<u16>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else {
        let vals = array.retrieve_array_subset::<Vec<f32>>(subset)?;
        Ok(vals)
    }
}
