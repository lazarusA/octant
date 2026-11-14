//! A GRIB2 file read as the NetCDF backend reads any file (`NcReader`): its messages
//! scanned once from their heads (`scan.rs`), laid out as variables (`layout.rs`), and a
//! window decoded from the messages it covers, each read whole and only when asked.

use std::collections::HashMap;

use gribberish::message::read_message;

use super::entry::{Entry, Item};
use super::layout::layout;
use super::scan::{Scan, scan};
use crate::data::backends::netcdf::pure::read_ahead::RangeSource;
use crate::data::backends::netcdf::reader::{NcReader, NcVar, Raw, failed};
use crate::data::blocks::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::i18n::DatasetKey;

pub struct GribReader {
    source: Box<dyn RangeSource + Send + Sync>,
    name: String,
    entries: Vec<Entry>,
}

impl GribReader {
    /// Scans the file `source` reads, `name` naming it in errors.
    pub fn open(
        source: Box<dyn RangeSource + Send + Sync>,
        name: &str,
    ) -> Result<Self, BlockStoreError> {
        let found = scan(source.as_ref(), name)?;
        Self::from_scan(source, name, found)
    }

    /// Reads the messages at `offsets` (its sidecar index's) of the file `source` reads.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_at(
        source: Box<dyn RangeSource + Send + Sync>,
        name: &str,
        offsets: &[u64],
    ) -> Result<Self, BlockStoreError> {
        let found = super::scan::scan_at(source.as_ref(), offsets, name)?;
        Self::from_scan(source, name, found)
    }

    fn from_scan(
        source: Box<dyn RangeSource + Send + Sync>,
        name: &str,
        found: Scan,
    ) -> Result<Self, BlockStoreError> {
        if found.fields.is_empty() {
            return Err(format!("'{name}' holds no GRIB2 message Octant can read").into());
        }
        let (entries, placed_twice) = layout(&found.fields)?;
        let left_out: Vec<&u64> = found.unreadable.iter().chain(&placed_twice).collect();
        if !left_out.is_empty() {
            let offsets: Vec<String> = left_out.iter().map(|o| format!("byte {o}")).collect();
            crate::data::backends::netcdf::pure::report_left_out(
                DatasetKey::GribMessagesLeftOut,
                &offsets,
            );
        }
        Ok(Self {
            entries,
            source,
            name: name.to_owned(),
        })
    }

    fn entry(&self, path: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.var.path == path)
    }

    /// The values of the message at `offset`, `len` bytes long: a `ny` by `nx` plane in
    /// stored order, every row scanning as the first does. gribberish reserves the points a
    /// message declares before decoding, so more than `MAX_DECODED_BYTES` of them (a damaged
    /// or hostile header) are refused, never reserved, as are messages longer than
    /// `MAX_MESSAGE_BYTES`.
    fn decode(
        &self,
        offset: u64,
        len: u64,
        (ny, nx): (usize, usize),
    ) -> Result<Vec<f64>, BlockStoreError> {
        let what = || format!("Failed reading '{}' at byte {offset}", self.name);
        let cap = super::sections::MAX_POINTS;
        let points = ny.checked_mul(nx).filter(|&n| n as u64 <= cap);
        let points = points.ok_or_else(|| failed(&what(), format!("{ny} x {nx} points")))?;
        if len > super::sections::MAX_MESSAGE_BYTES {
            return Err(failed(&what(), format!("it is {len} bytes long")));
        }
        let size = usize::try_from(len).map_err(|e| failed(&what(), e))?;
        let mut bytes = vec![0; size];
        (self.source.read_direct(offset, &mut bytes)).map_err(|e| failed(&what(), e))?;
        let declared = super::sections::declared_points(&bytes);
        if declared.is_some_and(|n| n > cap) {
            return Err(failed(&what(), format!("it declares {declared:?} points")));
        }
        if !super::sections::fit_templates(&bytes) {
            return Err(failed(&what(), "a section is shorter than its template"));
        }
        let message = read_message(&bytes, 0).ok_or_else(|| failed(&what(), "not a message"))?;
        let mut values = message.data().map_err(|e| failed(&what(), e))?;
        if values.len() != points {
            let held = format!("{} values, not {ny} x {nx}", values.len());
            return Err(failed(&what(), held));
        }
        if super::scanning::alternates(&bytes) {
            super::scanning::unreverse(&mut values, nx);
        }
        Ok(values)
    }

    /// The window `start..start + count` of a data variable, NaN where no message lies.
    fn read_data(
        &self,
        entry: &Entry,
        messages: &[Option<(u64, u64)>],
        (ny, nx): (usize, usize),
        start: &[usize],
        count: &[usize],
    ) -> Result<Vec<f32>, BlockStoreError> {
        let shape: Vec<usize> = entry.var.shape().collect();
        let window = Window::new(&shape, start, count)
            .ok_or_else(|| format!("Window outside '{}'", entry.var.path))?;
        let mut out = vec![f32::NAN; window.len];
        let (lead, plane) = (shape.len() - 2, window.plane);
        for (k, at) in window.planes().enumerate() {
            let Some(Some((offset, len))) = messages.get(at) else {
                continue;
            };
            let values = self.decode(*offset, *len, (ny, nx))?;
            let (y0, x0, cx) = (start[lead], start[lead + 1], count[lead + 1]);
            for r in 0..count[lead] {
                let src = &values[(y0 + r) * nx + x0..][..cx];
                let dst = &mut out[k * plane + r * cx..][..cx];
                for (d, s) in dst.iter_mut().zip(src) {
                    *d = *s as f32;
                }
            }
        }
        Ok(out)
    }
}

/// A window of a variable `[leading..., y, x]`: its size, its plane size, and the index of
/// each leading position it covers in the variable's messages.
struct Window {
    len: usize,
    plane: usize,
    /// The leading dimensions' strides through the messages, row-major.
    strides: Vec<usize>,
    lead_start: Vec<usize>,
    lead_count: Vec<usize>,
}

impl Window {
    fn new(shape: &[usize], start: &[usize], count: &[usize]) -> Option<Self> {
        let rank = shape.len();
        let inside = |d: usize| {
            start[d]
                .checked_add(count[d])
                .is_some_and(|e| e <= shape[d])
        };
        if rank < 2 || start.len() != rank || count.len() != rank || !(0..rank).all(inside) {
            return None;
        }
        let len = count.iter().try_fold(1usize, |a, &c| a.checked_mul(c))?;
        let lead = rank - 2;
        Some(Self {
            len,
            plane: count[lead] * count[lead + 1],
            strides: OctantBlock::row_major_strides(&shape[..lead]),
            lead_start: start[..lead].to_vec(),
            lead_count: count[..lead].to_vec(),
        })
    }

    /// The message index of each plane in the window, in row-major order.
    fn planes(&self) -> impl Iterator<Item = usize> + '_ {
        let total: usize = self.lead_count.iter().product();
        (0..if self.plane == 0 { 0 } else { total }).map(move |mut k| {
            let mut at = 0;
            for d in (0..self.strides.len()).rev() {
                at += (self.lead_start[d] + k % self.lead_count[d]) * self.strides[d];
                k /= self.lead_count[d];
            }
            at
        })
    }
}

impl NcReader for GribReader {
    fn global_attributes(&self) -> HashMap<String, String> {
        HashMap::from([("source".to_owned(), "GRIB2".to_owned())])
    }

    fn variables(&self) -> Vec<NcVar> {
        self.entries.iter().map(|e| e.var.clone()).collect()
    }

    fn variable(&self, path: &str) -> Option<NcVar> {
        self.entry(path).map(|e| e.var.clone())
    }

    fn read(&self, var: &NcVar, start: &[usize], count: &[usize]) -> Result<Raw, BlockStoreError> {
        let entry = self
            .entry(&var.path)
            .ok_or_else(|| format!("No variable '{}' in '{}'", var.path, self.name))?;
        match &entry.item {
            Item::Data { messages, ny, nx } => {
                (self.read_data(entry, messages, (*ny, *nx), start, count)).map(Raw::F32)
            }
            Item::Values(values) => {
                let (s, c) = (start.first().copied(), count.first().copied());
                let window = s.zip(c).and_then(|(s, c)| values.get(s..s.checked_add(c)?));
                let window = window.ok_or_else(|| format!("Window outside '{}'", var.path))?;
                Ok(Raw::F64(window.to_vec()))
            }
            Item::Scalar => Ok(Raw::F64(vec![0.0])),
        }
    }

    fn labels(&self, _var: &NcVar) -> Option<Vec<String>> {
        None
    }
}
