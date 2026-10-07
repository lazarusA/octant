//! Dtype conversion, calibration, and array subset retrieval as f32 through an optional chunk cache.

use crate::data::DataCalibration;
use crate::data::blocks::BlockStoreError;
use zarrs::array::chunk_cache::{ChunkCache, ChunkCacheDecodedLruSizeLimit};
use zarrs::array::data_type::*;
use zarrs::array::{Array, ArraySubset, CodecOptions, DataType, FromArrayBytes};
use zarrs::storage::ReadableStorageTraits;

trait SubsetRetriever {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError>;
}

impl<TStorage: ?Sized + ReadableStorageTraits + 'static> SubsetRetriever for Array<TStorage> {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError> {
        self.retrieve_array_subset(subset)
            .map_err(|e| format!("{e}").into())
    }
}

impl SubsetRetriever for ChunkCacheDecodedLruSizeLimit {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError> {
        self.retrieve_array_subset(subset, &CodecOptions::default())
            .map_err(|e| format!("{e}").into())
    }
}

fn decode_with<R: SubsetRetriever>(
    retriever: &R,
    dt: &DataType,
    calibration: &DataCalibration,
    subset: &ArraySubset,
) -> Result<Vec<f32>, BlockStoreError> {
    let has_tx = calibration.has_transformation();

    macro_rules! decode_typed {
        ($t:ty) => {{
            let vals: Vec<$t> = retriever.retrieve_subset(subset)?;
            if !has_tx {
                Ok(vals.into_iter().map(|v| v as f32).collect())
            } else {
                Ok(vals
                    .into_iter()
                    .map(|v| calibration.transform(v as f64))
                    .collect())
            }
        }};
    }

    if dt.is::<Float32DataType>() {
        let mut raw_vals: Vec<f32> = retriever.retrieve_subset(subset)?;
        if has_tx {
            calibration.transform_slice_in_place(&mut raw_vals);
        }
        Ok(raw_vals)
    } else if dt.is::<Float64DataType>() {
        decode_typed!(f64)
    } else if dt.is::<Int32DataType>() {
        decode_typed!(i32)
    } else if dt.is::<Int16DataType>() {
        decode_typed!(i16)
    } else if dt.is::<Int8DataType>() {
        decode_typed!(i8)
    } else if dt.is::<UInt32DataType>() {
        decode_typed!(u32)
    } else if dt.is::<UInt16DataType>() {
        decode_typed!(u16)
    } else if dt.is::<UInt8DataType>() {
        decode_typed!(u8)
    } else if dt.is::<Int64DataType>() {
        decode_typed!(i64)
    } else if dt.is::<UInt64DataType>() {
        decode_typed!(u64)
    } else {
        decode_other(retriever, dt, calibration, subset)
    }
}

/// Booleans as 0 or 1, half-precision floats widened, anything else read as `f32`.
fn decode_other<R: SubsetRetriever>(
    retriever: &R,
    dt: &DataType,
    calibration: &DataCalibration,
    subset: &ArraySubset,
) -> Result<Vec<f32>, BlockStoreError> {
    let widen = |vals: Vec<f64>| -> Vec<f32> {
        if calibration.has_transformation() {
            vals.into_iter().map(|v| calibration.transform(v)).collect()
        } else {
            vals.into_iter().map(|v| v as f32).collect()
        }
    };
    if dt.is::<BoolDataType>() {
        let vals: Vec<u8> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| f32::from(v != 0)).collect())
    } else if dt.is::<Float16DataType>() {
        let vals: Vec<half::f16> = retriever.retrieve_subset(subset)?;
        Ok(widen(vals.into_iter().map(f64::from).collect()))
    } else if dt.is::<BFloat16DataType>() {
        let vals: Vec<half::bf16> = retriever.retrieve_subset(subset)?;
        Ok(widen(vals.into_iter().map(f64::from).collect()))
    } else {
        retriever.retrieve_subset(subset)
    }
}

/// Dtype-conversion and calibration helper for reading array subsets as f32 through an optional chunk cache.
pub fn retrieve_array_subset_as_f32<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    cache: Option<&ChunkCacheDecodedLruSizeLimit>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, BlockStoreError> {
    let dt = array.data_type();
    let calibration = DataCalibration::from_json_map(array.attributes());
    match cache {
        Some(c) => decode_with(c, dt, &calibration, subset),
        None => decode_with(array, dt, &calibration, subset),
    }
}

/// Reads an array subset as `f64`, applying CF scale, offset and masking, for coordinates
/// that need more precision than `f32`. Fails for non-numeric data types.
pub fn retrieve_array_subset_as_f64<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    subset: &ArraySubset,
) -> Result<Vec<f64>, BlockStoreError> {
    let dt = array.data_type();
    macro_rules! widen {
        ($t:ty) => {{
            let vals: Vec<$t> = array.retrieve_subset(subset)?;
            vals.into_iter().map(|v| v as f64).collect::<Vec<f64>>()
        }};
    }
    let mut values = if dt.is::<Float64DataType>() {
        array.retrieve_subset::<Vec<f64>>(subset)?
    } else if dt.is::<Float32DataType>() {
        widen!(f32)
    } else if dt.is::<Int64DataType>() {
        widen!(i64)
    } else if dt.is::<Int32DataType>() {
        widen!(i32)
    } else if dt.is::<Int16DataType>() {
        widen!(i16)
    } else if dt.is::<Int8DataType>() {
        widen!(i8)
    } else if dt.is::<UInt64DataType>() {
        widen!(u64)
    } else if dt.is::<UInt32DataType>() {
        widen!(u32)
    } else if dt.is::<UInt16DataType>() {
        widen!(u16)
    } else if dt.is::<UInt8DataType>() {
        widen!(u8)
    } else if dt.is::<Float16DataType>() {
        let vals: Vec<half::f16> = array.retrieve_subset(subset)?;
        vals.into_iter().map(f64::from).collect()
    } else if dt.is::<BFloat16DataType>() {
        let vals: Vec<half::bf16> = array.retrieve_subset(subset)?;
        vals.into_iter().map(f64::from).collect()
    } else {
        return Err("coordinate data type is not numeric".into());
    };
    let calibration = DataCalibration::from_json_map(array.attributes());
    if calibration.has_transformation() {
        for v in &mut values {
            *v = calibration.transform_f64(*v);
        }
    }
    Ok(values)
}
