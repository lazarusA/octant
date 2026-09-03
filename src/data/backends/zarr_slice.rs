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
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
    if dt.is::<Float32DataType>() {
        retriever.retrieve_subset(subset)
    } else if dt.is::<Float64DataType>() {
        let vals: Vec<f64> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<Int32DataType>() {
        let vals: Vec<i32> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<Int64DataType>() {
        let vals: Vec<i64> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<UInt32DataType>() {
        let vals: Vec<u32> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<UInt64DataType>() {
        let vals: Vec<u64> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<Int16DataType>() {
        let vals: Vec<i16> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<UInt16DataType>() {
        let vals: Vec<u16> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<Int8DataType>() {
        let vals: Vec<i8> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt.is::<UInt8DataType>() {
        let vals: Vec<u8> = retriever.retrieve_subset(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
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

/// Dtype-conversion helper for reading array subsets as f32 through an optional chunk cache.
pub fn retrieve_array_subset_as_f32<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    cache: Option<&ChunkCacheDecodedLruSizeLimit>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
    let dt = array.data_type();
    match cache {
        Some(c) => decode_with(c, dt, subset),
        None => decode_with(array, dt, subset),
    }
}
