use std::error::Error;
use zarrs::array::chunk_cache::{ChunkCache, ChunkCacheDecodedLruSizeLimit};
use zarrs::array::data_type::*;
use zarrs::array::{Array, ArraySubset, CodecOptions};
use zarrs::storage::ReadableStorageTraits;

/// Dtype-conversion helper for reading array subsets as f32 through an optional chunk cache.
pub fn retrieve_array_subset_as_f32<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    cache: Option<&ChunkCacheDecodedLruSizeLimit>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
    let dt = array.data_type();
    let opt = CodecOptions::default();

    if let Some(c) = cache {
        if dt.is::<Float32DataType>() {
            let vals: Vec<f32> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals)
        } else if dt.is::<Float64DataType>() {
            let vals: Vec<f64> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int32DataType>() {
            let vals: Vec<i32> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int64DataType>() {
            let vals: Vec<i64> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt32DataType>() {
            let vals: Vec<u32> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt64DataType>() {
            let vals: Vec<u64> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int16DataType>() {
            let vals: Vec<i16> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt16DataType>() {
            let vals: Vec<u16> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int8DataType>() {
            let vals: Vec<i8> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt8DataType>() {
            let vals: Vec<u8> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<BoolDataType>() {
            let vals: Vec<u8> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals
                .into_iter()
                .map(|v| if v != 0 { 1.0 } else { 0.0 })
                .collect())
        } else {
            let vals: Vec<f32> = c.retrieve_array_subset(subset, &opt)?;
            Ok(vals)
        }
    } else {
        if dt.is::<Float32DataType>() {
            let vals: Vec<f32> = array.retrieve_array_subset(subset)?;
            Ok(vals)
        } else if dt.is::<Float64DataType>() {
            let vals: Vec<f64> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int32DataType>() {
            let vals: Vec<i32> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int64DataType>() {
            let vals: Vec<i64> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt32DataType>() {
            let vals: Vec<u32> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt64DataType>() {
            let vals: Vec<u64> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int16DataType>() {
            let vals: Vec<i16> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt16DataType>() {
            let vals: Vec<u16> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<Int8DataType>() {
            let vals: Vec<i8> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<UInt8DataType>() {
            let vals: Vec<u8> = array.retrieve_array_subset(subset)?;
            Ok(vals.into_iter().map(|v| v as f32).collect())
        } else if dt.is::<BoolDataType>() {
            let vals: Vec<u8> = array.retrieve_array_subset(subset)?;
            Ok(vals
                .into_iter()
                .map(|v| if v != 0 { 1.0 } else { 0.0 })
                .collect())
        } else {
            let vals: Vec<f32> = array.retrieve_array_subset(subset)?;
            Ok(vals)
        }
    }
}
