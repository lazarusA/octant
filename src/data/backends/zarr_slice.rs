use crate::data::DataCalibration;
use std::error::Error;
use zarrs::array::chunk_cache::{ChunkCache, ChunkCacheDecodedLruSizeLimit};
use zarrs::array::data_type::*;
use zarrs::array::{Array, ArraySubset, CodecOptions, DataType, FromArrayBytes};
use zarrs::storage::ReadableStorageTraits;

trait SubsetRetriever {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, Box<dyn Error + Send + Sync>>;
}

impl<TStorage: ?Sized + ReadableStorageTraits + 'static> SubsetRetriever for Array<TStorage> {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        Ok(self.retrieve_array_subset(subset)?)
    }
}

impl SubsetRetriever for ChunkCacheDecodedLruSizeLimit {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        Ok(self.retrieve_array_subset(subset, &CodecOptions::default())?)
    }
}

fn decode_with<R: SubsetRetriever>(
    retriever: &R,
    dt: &DataType,
    calibration: &DataCalibration,
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
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
    } else if dt.is::<BoolDataType>() {
        let vals: Vec<u8> = retriever.retrieve_subset(subset)?;
        Ok(vals
            .into_iter()
            .map(|v| if v != 0 { 1.0 } else { 0.0 })
            .collect())
    } else {
        retriever.retrieve_subset(subset)
    }
}

/// Dtype-conversion and calibration helper for reading array subsets as f32 through an optional chunk cache.
pub fn retrieve_array_subset_as_f32<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    cache: Option<&ChunkCacheDecodedLruSizeLimit>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
    let dt = array.data_type();
    let calibration = DataCalibration::from_json_map(array.attributes());
    match cache {
        Some(c) => decode_with(c, dt, &calibration, subset),
        None => decode_with(array, dt, &calibration, subset),
    }
}
