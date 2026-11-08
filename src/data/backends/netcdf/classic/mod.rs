//! The reader of classic NetCDF files (CDF-1, CDF-2, CDF-5) over any byte [`RangeSource`], for
//! the browser's data workers and remote files on the desktop (netCDF-C reads local ones).
//! Fixed-size variables are stored whole at their `begin`; record variables (first
//! dimension unlimited) hold one slab per record, interleaved every `recsize` bytes.

mod header;
mod layout;
mod values;

use std::collections::HashMap;

use super::coord_read::char_labels;
use super::pure::read_ahead::{BLOCK, RangeSource};
use super::reader::{NcReader, NcVar, Raw, failed};
use crate::data::blocks::BlockStoreError;
use header::{Header, NcType, Short};

pub use header::is_classic;

/// The first read of the header; larger headers are read again at twice the size, up to
/// `MAX_HEADER`, so a corrupt length never reads a whole file.
const HEADER_READ: u64 = 64 << 10;
const MAX_HEADER: u64 = 64 << 20;

pub struct ClassicReader {
    source: Box<dyn RangeSource + Send + Sync>,
    header: Header,
    vars: Vec<NcVar>,
    /// Bytes from one record to the next.
    recsize: u64,
}

/// The header of the file `source` reads, reading more until it is whole.
fn read_header(source: &dyn RangeSource, name: &str) -> Result<Header, BlockStoreError> {
    let mut size = HEADER_READ;
    loop {
        let mut bytes = vec![0; size.min(source.len()) as usize];
        source
            .read_at(0, &mut bytes)
            .map_err(|e| failed(&format!("Failed to read '{name}'"), e))?;
        match header::parse(&bytes, source.len()) {
            Ok(header) => return Ok(header),
            Err(Short::More) if size < source.len().min(MAX_HEADER) => size *= 2,
            Err(Short::More) => return Err(format!("'{name}' has no readable header").into()),
            Err(Short::Bad(why)) => return Err(failed(&format!("Bad header in '{name}'"), why)),
        }
    }
}

impl ClassicReader {
    pub fn open(
        source: Box<dyn RangeSource + Send + Sync>,
        name: &str,
    ) -> Result<Self, BlockStoreError> {
        let header = read_header(source.as_ref(), name)?;
        let bad = |e: String| failed(&format!("Bad header in '{name}'"), e);
        let (recsize, numrecs) = layout::records(&header, source.len()).map_err(bad)?;
        let vars = layout::describe(&header, numrecs).map_err(bad)?;
        Ok(Self {
            source,
            header,
            vars,
            recsize,
        })
    }

    /// The bytes of the window `start..start + count` of the variable at `index`, read a
    /// run at a time: the dimensions read whole at the end run together within a record.
    fn window(
        &self,
        index: usize,
        start: &[usize],
        count: &[usize],
    ) -> Result<Vec<u8>, BlockStoreError> {
        let var = &self.header.vars[index];
        let shape: Vec<u64> = self.vars[index].shape().map(|n| n as u64).collect();
        let rank = shape.len();
        let inside =
            |d: usize| (start[d].checked_add(count[d])).is_some_and(|e| e as u64 <= shape[d]);
        if start.len() != rank || count.len() != rank || !(0..rank).all(inside) {
            return Err(format!("Window outside '{}'", var.name).into());
        }
        let size = var.ty.size();
        // The window must fit in memory (32 bits in the browser), though its variable may not.
        let bytes = (count.iter())
            .try_fold(size, |a, &c| a.checked_mul(c as u64))
            .and_then(|b| usize::try_from(b).ok())
            .ok_or_else(|| format!("Window of '{}' too large to read here", var.name))?;
        if bytes == 0 {
            return Ok(Vec::new());
        }
        let record = layout::is_record(&self.header, var);
        let r = layout::runs(&shape, start, count, record);
        let (k, run, inner) = (r.k, r.run, r.inner);
        let step = |d: usize| match record && d == 0 {
            true => self.recsize,
            false => r.strides[d].saturating_mul(size),
        };
        // A run lies within the window, whose bytes fit `usize`.
        let run_bytes = (run * size) as usize;
        let mut out = vec![0; bytes];
        let mut at = start[..k].to_vec();
        for buf in out.chunks_exact_mut(run_bytes) {
            let outer = (at.iter().enumerate())
                .map(|(d, &i)| (i as u64).saturating_mul(step(d)))
                .fold(var.begin, u64::saturating_add);
            let offset = outer.saturating_add(inner.saturating_mul(size));
            // Records a block or more apart share no block: each read fetches its own bytes.
            let read = match record && self.recsize >= BLOCK {
                true => self.source.read_direct(offset, buf),
                false => self.source.read_at(offset, buf),
            };
            read.map_err(|e| failed(&format!("Failed reading '{}'", var.name), e))?;
            // The next run: the outer dimensions counted up, the last fastest.
            let Some(d) = (0..k).rev().find(|&d| at[d] + 1 < start[d] + count[d]) else {
                break;
            };
            at[d] += 1;
            at[d + 1..].copy_from_slice(&start[d + 1..k]);
        }
        Ok(out)
    }

    fn index_of(&self, var: &NcVar) -> Result<usize, BlockStoreError> {
        (self.vars.iter().position(|v| v.path == var.path))
            .ok_or_else(|| format!("Variable '{}' not found", var.path).into())
    }
}

impl NcReader for ClassicReader {
    fn global_attributes(&self) -> HashMap<String, String> {
        (self.header.attrs.iter())
            .map(|a| (a.name.clone(), values::text(a)))
            .collect()
    }

    fn variables(&self) -> Vec<NcVar> {
        self.vars.clone()
    }

    fn variable(&self, path: &str) -> Option<NcVar> {
        let path = path.trim_matches('/');
        self.vars.iter().find(|v| v.path == path).cloned()
    }

    fn read(&self, var: &NcVar, start: &[usize], count: &[usize]) -> Result<Raw, BlockStoreError> {
        let index = self.index_of(var)?;
        let bytes = self.window(index, start, count)?;
        Ok(match self.header.vars[index].ty {
            NcType::Float => Raw::F32(values::floats(&bytes)),
            ty => Raw::F64(values::numbers(ty, &bytes)),
        })
    }

    fn labels(&self, var: &NcVar) -> Option<Vec<String>> {
        let [(_, rows), (_, width)] = var.dims.as_slice() else {
            return None;
        };
        if var.data_type != "char" || *width == 0 {
            return None;
        }
        let bytes = self
            .window(self.index_of(var).ok()?, &[0, 0], &[*rows, *width])
            .ok()?;
        let labels = char_labels(&bytes, *rows, *width);
        (!labels.is_empty()).then_some(labels)
    }
}
