//! Big-endian values of a classic NetCDF file: as numbers for reads, and attributes as the
//! text netCDF-C prints for them (one value alone, several as a list, `char` as text).

use super::header::{Attr, NcType};
use crate::data::DataCalibration;
use crate::data::backends::netcdf::reader::list;

/// `bytes` of type `ty` as `f64`s (`char` as its byte values).
pub fn numbers(ty: NcType, bytes: &[u8]) -> Vec<f64> {
    use NcType as T;
    macro_rules! each {
        ($n:literal, $t:ty) => {
            (bytes.as_chunks::<$n>().0.iter())
                .map(|c| <$t>::from_be_bytes(*c) as f64)
                .collect()
        };
    }
    match ty {
        T::Byte => bytes.iter().map(|&b| f64::from(b as i8)).collect(),
        T::Char | T::UByte => bytes.iter().map(|&b| f64::from(b)).collect(),
        T::Short => each!(2, i16),
        T::UShort => each!(2, u16),
        T::Int => each!(4, i32),
        T::UInt => each!(4, u32),
        T::Float => each!(4, f32),
        T::Double => each!(8, f64),
        T::Int64 => each!(8, i64),
        T::UInt64 => each!(8, u64),
    }
}

/// `bytes` of a `float` variable as `f32`s.
pub fn floats(bytes: &[u8]) -> Vec<f32> {
    (bytes.as_chunks::<4>().0.iter())
        .map(|c| f32::from_be_bytes(*c))
        .collect()
}

/// The text netCDF-C shows for `attr`.
pub fn text(attr: &Attr) -> String {
    use NcType as T;
    let b = &attr.bytes;
    macro_rules! typed {
        ($n:literal, $t:ty) => {{
            let values: Vec<$t> = (b.as_chunks::<$n>().0.iter())
                .map(|c| <$t>::from_be_bytes(*c))
                .collect();
            list(&values)
        }};
    }
    match attr.ty {
        // Writers pad text with NULs, which netCDF-C leaves out.
        T::Char => String::from_utf8_lossy(b).trim_end_matches('\0').to_owned(),
        T::Byte => list(&b.iter().map(|&x| x as i8).collect::<Vec<_>>()),
        T::UByte => list(b),
        T::Short => typed!(2, i16),
        T::UShort => typed!(2, u16),
        T::Int => typed!(4, i32),
        T::UInt => typed!(4, u32),
        T::Float => typed!(4, f32),
        T::Double => typed!(8, f64),
        T::Int64 => typed!(8, i64),
        T::UInt64 => typed!(8, u64),
    }
}

/// The first number of `attr`, or text that parses as one.
fn number(attr: &Attr) -> Option<f64> {
    match attr.ty {
        NcType::Char => text(attr).trim().parse().ok(),
        ty => numbers(ty, &attr.bytes).first().copied(),
    }
}

/// CF scale, offset, fill and valid range from `attrs`, read as netCDF-C's reader does.
pub fn calibration(attrs: &[Attr]) -> DataCalibration {
    DataCalibration::from_numbers(|name| attrs.iter().find(|a| a.name == name).and_then(number))
}
