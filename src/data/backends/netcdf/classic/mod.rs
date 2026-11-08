//! The reader of classic NetCDF files (CDF-1, CDF-2, CDF-5) over any byte [`Source`], for
//! the browser's data workers and remote files on the desktop (netCDF-C reads local ones).
//! Fixed-size variables are stored whole at their `begin`; record variables (first
//! dimension unlimited) hold one slab per record, interleaved every `recsize` bytes.

mod header;
mod layout;
mod values;

use std::collections::HashMap;

use hdf5_pure::Source;

use super::coord_read::{c_text, clean_label};
use super::reader::{NcReader, NcVar, Raw};
use crate::data::blocks::BlockStoreError;
use header::{Header, NcType, Short};
use layout::Runs;

pub use header::is_classic;

/// The first read of the header; larger headers are read again at twice the size.
const HEADER_READ: u64 = 64 << 10;

pub struct ClassicReader {
    source: Box<dyn Source + Send + Sync>,
    header: Header,
    vars: Vec<NcVar>,
    /// Bytes from one record to the next.
    recsize: u64,
}

fn failed(what: &str, e: impl std::fmt::Display) -> BlockStoreError {
    format!("{what}: {e}").into()
}

/// The header of the file `source` reads, reading more until it is whole.
fn read_header(source: &dyn Source, name: &str) -> Result<Header, BlockStoreError> {
    let mut size = HEADER_READ;
    loop {
        let mut bytes = vec![0; size.min(source.len()) as usize];
        source
            .read_at(0, &mut bytes)
            .map_err(|e| failed(&format!("Failed to read '{name}'"), e))?;
        match header::parse(&bytes) {
            Ok(header) => return Ok(header),
            Err(Short::More) if size < source.len() => size = size.saturating_mul(2),
            Err(Short::More) => return Err(format!("'{name}' ends inside its header").into()),
            Err(Short::Bad(why)) => return Err(failed(&format!("Bad header in '{name}'"), why)),
        }
    }
}

impl ClassicReader {
    pub fn open(
        source: Box<dyn Source + Send + Sync>,
        name: &str,
    ) -> Result<Self, BlockStoreError> {
        let header = read_header(source.as_ref(), name)?;
        let (recsize, numrecs) = layout::records(&header, source.len());
        let vars = layout::describe(&header, numrecs);
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
        if start.len() != rank || count.len() != rank {
            return Err(format!("Window of rank {} for '{}'", start.len(), var.name).into());
        }
        let total = (count.iter())
            .try_fold(1u64, |a, &c| a.checked_mul(c as u64))
            .ok_or("Window too large")?;
        if total == 0 {
            return Ok(Vec::new());
        }
        let size = var.ty.size();
        let record = layout::is_record(&self.header, var);
        let Runs {
            k,
            run,
            inner,
            strides,
        } = layout::runs(&shape, start, count, record);
        let step = |d: usize| match record && d == 0 {
            true => self.recsize,
            false => strides[d].saturating_mul(size),
        };
        let mut out = Vec::with_capacity(total.saturating_mul(size) as usize);
        let mut at = start[..k].to_vec();
        loop {
            let outer = (at.iter().enumerate())
                .map(|(d, &i)| (i as u64).saturating_mul(step(d)))
                .fold(var.begin, u64::saturating_add);
            let mut buf = vec![0; run.saturating_mul(size) as usize];
            self.source
                .read_at(outer.saturating_add(inner.saturating_mul(size)), &mut buf)
                .map_err(|e| failed(&format!("Failed reading '{}'", var.name), e))?;
            out.extend_from_slice(&buf);
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
        let labels: Vec<String> = (bytes.chunks(*width))
            .map(|row| clean_label(&String::from_utf8_lossy(c_text(row))))
            .collect();
        (!labels.is_empty()).then_some(labels)
    }
}
