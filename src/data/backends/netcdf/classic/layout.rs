//! Where a classic file keeps its values: record variables (first dimension unlimited) one
//! slab per record, every `recsize` bytes; and how a window is read as contiguous runs.

use super::header::{Header, Var};
use super::values;
use crate::data::backends::netcdf::reader::NcVar;

/// Whether `var`'s first dimension is the record (unlimited) dimension.
pub fn is_record(header: &Header, var: &Var) -> bool {
    var.dimids.first().is_some_and(|&d| header.dims[d].1 == 0)
}

/// Bytes from one record to the next, and the record count (from a file `len` bytes
/// long when the header leaves it out, as while a writer streams records).
pub fn records(header: &Header, len: u64) -> Result<(u64, u64), String> {
    let slab = |v: &Var| -> Option<u64> {
        let mut fixed = v.dimids.iter().skip(1).map(|&d| header.dims[d].1);
        fixed.try_fold(v.ty.size(), u64::checked_mul)
    };
    let records: Vec<&Var> = header
        .vars
        .iter()
        .filter(|v| is_record(header, v))
        .collect();
    // One record variable is stored unpadded; several are each padded to 4 bytes.
    let recsize = match records.as_slice() {
        [one] => slab(one),
        many => {
            (many.iter()).try_fold(0u64, |sum, v| sum.checked_add(slab(v)?.next_multiple_of(4)))
        }
    }
    .ok_or("Records too large")?;
    let numrecs = header.numrecs.unwrap_or_else(|| {
        let first = records.iter().map(|v| v.begin).min().unwrap_or(len);
        len.saturating_sub(first).checked_div(recsize).unwrap_or(0)
    });
    Ok((recsize, numrecs))
}

/// Every variable as the reader describes it, the record dimension `numrecs` long; an
/// error for one too large to address here.
pub fn describe(header: &Header, numrecs: u64) -> Result<Vec<NcVar>, String> {
    let dim = |d: usize| {
        let (name, len) = &header.dims[d];
        let len = if *len == 0 { numrecs } else { *len };
        Some((name.clone(), usize::try_from(len).ok()?))
    };
    (header.vars.iter())
        .map(|v| {
            // Sizes fit `usize`, so the window arithmetic in `runs` cannot overflow.
            let dims: Option<Vec<(String, usize)>> = v.dimids.iter().map(|&d| dim(d)).collect();
            let dims = (dims.filter(|dims| {
                let elem = v.ty.size() as usize; // at most 8
                (dims.iter())
                    .try_fold(elem, |a, (_, n)| a.checked_mul(*n))
                    .is_some()
            }))
            .ok_or_else(|| format!("'{}' is too large", v.name))?;
            Ok(NcVar {
                path: v.name.clone(),
                data_type: v.ty.name(),
                dims,
                attributes: (v.attrs.iter())
                    .map(|a| (a.name.clone(), values::text(a)))
                    .collect(),
                calibration: values::calibration(&v.attrs),
            })
        })
        .collect()
}

/// How a window is read: one run of `run` elements, starting `inner` elements into the
/// dimensions `k..`, per index of the outer dimensions `..k`.
pub struct Runs {
    pub k: usize,
    pub run: u64,
    pub inner: u64,
    /// Element strides within one record (record variables) or the whole variable.
    pub strides: Vec<u64>,
}

/// The runs of the window `start..start + count` of a variable shaped `shape`.
pub fn runs(shape: &[u64], start: &[usize], count: &[usize], record: bool) -> Runs {
    let rank = shape.len();
    let mut strides = vec![1u64; rank];
    for i in (0..rank.saturating_sub(1)).rev() {
        strides[i] = strides[i + 1].saturating_mul(shape[i + 1]);
    }
    // A run covers dimensions `k..`: all but its first read whole, never across records.
    let mut k = rank;
    while k > usize::from(record) && (k == rank || count[k] as u64 == shape[k]) {
        k -= 1;
    }
    let (run, inner) = match k < rank {
        true => (
            (count[k] as u64).saturating_mul(strides[k]),
            (start[k] as u64).saturating_mul(strides[k]),
        ),
        false => (1, 0),
    };
    Runs {
        k,
        run,
        inner,
        strides,
    }
}
