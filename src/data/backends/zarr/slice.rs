//! Dtype conversion, calibration, and array subset retrieval as f32 through an optional chunk cache.

use crate::data::DataCalibration;
use crate::data::blocks::BlockStoreError;
use zarrs::array::chunk_cache::{ChunkCache, ChunkCacheDecodedLruSizeLimit};
use zarrs::array::data_type::*;
use zarrs::array::{Array, ArraySubset, CodecOptions, DataType, FromArrayBytes};
use zarrs::array::{ArrayShardedReadableExt, ArrayShardedReadableExtCache};
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

/// How an array's subsets are read.
#[derive(Clone, Copy)]
pub enum ChunkReader<'a> {
    /// Straight from the store, whole chunks at a time.
    Direct,
    /// Through a cache of decoded chunks.
    Cached(&'a ChunkCacheDecodedLruSizeLimit),
    /// A sharded array: only the inner chunks a subset touches, located
    /// through the shard indexes the cache keeps.
    Sharded(&'a ArrayShardedReadableExtCache),
}

/// A sharded array read through its shard indexes.
struct Sharded<'a, TStorage: ?Sized> {
    array: &'a Array<TStorage>,
    cache: &'a ArrayShardedReadableExtCache,
}

impl<TStorage: ?Sized + ReadableStorageTraits + 'static> SubsetRetriever for Sharded<'_, TStorage> {
    fn retrieve_subset<T: FromArrayBytes>(
        &self,
        subset: &ArraySubset,
    ) -> Result<T, BlockStoreError> {
        self.array
            .retrieve_array_subset_sharded_opt(self.cache, subset, &CodecOptions::default())
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

/// Dtype-conversion and calibration helper for reading array subsets as f32 through `reader`.
pub fn retrieve_array_subset_as_f32<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    reader: ChunkReader<'_>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, BlockStoreError> {
    let dt = array.data_type();
    let calibration = DataCalibration::from_json_map(array.attributes());
    match reader {
        ChunkReader::Direct => decode_with(array, dt, &calibration, subset),
        ChunkReader::Cached(c) => decode_with(c, dt, &calibration, subset),
        ChunkReader::Sharded(cache) => {
            decode_with(&Sharded { array, cache }, dt, &calibration, subset)
        }
    }
}

/// Reads an integer array subset as label ids, kept exact: each value's low
/// 32 bits are stored as the bits of an `f32` (read back with `f32::to_bits`),
/// never converted. 64-bit ids beyond 32 bits share colors with the ids of
/// their low bits, reported once. `None` for non-integer data types.
pub fn retrieve_array_subset_as_label_bits<TStorage: ?Sized + ReadableStorageTraits + 'static>(
    array: &Array<TStorage>,
    reader: ChunkReader<'_>,
    subset: &ArraySubset,
) -> Result<Option<Vec<f32>>, BlockStoreError> {
    fn bits<R: SubsetRetriever>(
        retriever: &R,
        dt: &DataType,
        subset: &ArraySubset,
    ) -> Result<Option<Vec<f32>>, BlockStoreError> {
        macro_rules! ids {
            ($t:ty) => {
                ids!($t, |_| false)
            };
            // `$wide`: whether an id does not fit 32 bits (64-bit types).
            ($t:ty, $wide:expr) => {{
                let vals: Vec<$t> = retriever.retrieve_subset(subset)?;
                if vals.iter().copied().any($wide) {
                    report_wide_label_ids();
                }
                Ok(Some(
                    vals.into_iter().map(|v| f32::from_bits(v as u32)).collect(),
                ))
            }};
        }
        if dt.is::<UInt8DataType>() {
            ids!(u8)
        } else if dt.is::<UInt16DataType>() {
            ids!(u16)
        } else if dt.is::<UInt32DataType>() {
            ids!(u32)
        } else if dt.is::<UInt64DataType>() {
            ids!(u64, |v: u64| u32::try_from(v).is_err())
        } else if dt.is::<Int8DataType>() {
            ids!(i8)
        } else if dt.is::<Int16DataType>() {
            ids!(i16)
        } else if dt.is::<Int32DataType>() {
            ids!(i32)
        } else if dt.is::<Int64DataType>() {
            ids!(i64, |v: i64| u32::try_from(v).is_err())
        } else {
            Ok(None)
        }
    }
    let dt = array.data_type();
    match reader {
        ChunkReader::Direct => bits(array, dt, subset),
        ChunkReader::Cached(c) => bits(c, dt, subset),
        ChunkReader::Sharded(cache) => bits(&Sharded { array, cache }, dt, subset),
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

/// Reports, once per session, label ids that do not fit 32 bits.
fn report_wide_label_ids() {
    static REPORTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !REPORTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        crate::ui::toast::report(
            crate::ui::toast::Severity::Warning,
            crate::i18n::tr(crate::i18n::NoticesKey::LabelIdsBeyond32Bits),
            "",
        );
    }
}
