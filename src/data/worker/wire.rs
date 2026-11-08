//! Messages between the page (`pool.rs`) and a data worker (`host.rs`): a
//! JSON text, then the binary parts it names, sent as transferred typed
//! arrays: a block's values (`f32`), its value range and coordinates (`f64`,
//! which JSON cannot hold as NaN) and a local file's bytes.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use super::wire_meta::MetadataHeader;
use crate::data::DataSource;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::i18n::Language;
use crate::ui::toast::Severity;

/// From the page to a worker.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToWorker {
    /// The worker's share of the download budget, sent before any job.
    Budget { bytes: u64 },
    /// Load `slice` of `source`, answered by `Block` or `Failed` with `id`;
    /// `language` is the page's, for the texts the worker writes.
    Load {
        id: u64,
        source: DataSource,
        slice: SliceRequest,
        language: Language,
    },
    /// Inspect `source`, answered by `Inspected` or `Failed` with `id`: stores only a
    /// worker reads (NetCDF, `netcdf/wasm.rs`).
    Inspect {
        id: u64,
        source: DataSource,
        language: Language,
    },
    /// Drop the stores of `uri`: its dataset closed.
    Release { uri: String },
    /// The answer to the worker's `Read` with `id`: its bytes (or the `File`) follow, unless
    /// `error`.
    ReadDone { id: u64, error: Option<String> },
}

/// From a worker to the page.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum FromWorker {
    /// The worker listens: until then it drops what it is sent.
    Hello,
    /// A loaded block; its values and its `f64`s follow ([`BlockHeader::split`]).
    Block {
        id: u64,
        header: BlockHeader,
    },
    Failed {
        id: u64,
        error: String,
    },
    /// An inspected store; its coordinates' numbers follow ([`MetadataHeader::split`]).
    Inspected {
        id: u64,
        header: Box<MetadataHeader>,
    },
    /// What the worker's fetches counted since its last report (`blocks::progress::totals`).
    Progress {
        counts: [u64; 3],
    },
    /// A toast reported in the worker (`ui::toast::report`).
    Notice {
        severity: Severity,
        title: String,
        detail: String,
    },
    /// Read a local file (`local://`), which only the page holds.
    Read {
        id: u64,
        url: String,
        read: LocalRead,
    },
}

/// What a worker reads of a local file.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LocalRead {
    All,
    Range {
        offset: u64,
        length: u64,
    },
    /// Its size, as 8 little-endian bytes.
    Size,
    /// The `File` itself, for a reader that reads it synchronously (`FileReaderSync`).
    File,
}

/// A block without its numbers: those travel as typed arrays.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BlockHeader {
    variable_name: String,
    shape: Vec<usize>,
    dimension_names: Vec<String>,
    origin: Vec<usize>,
    attributes: HashMap<String, String>,
    flipped_dims: Vec<String>,
    /// Each coordinate's name and length, in the order the `f64`s hold them.
    coordinates: Vec<(String, usize)>,
}

impl BlockHeader {
    /// `block` as its header, its values and its `f64`s: the value range
    /// (minimum, maximum), then each coordinate.
    pub fn split(block: OctantBlock) -> (Self, Arc<[f32]>, Vec<f64>) {
        let count = block.coordinates.values().map(Vec::len).sum::<usize>();
        let mut numbers = Vec::with_capacity(count.saturating_add(2));
        numbers.extend([f64::from(block.min_value), f64::from(block.max_value)]);
        let mut coordinates = Vec::with_capacity(block.coordinates.len());
        for (name, values) in block.coordinates {
            coordinates.push((name, values.len()));
            numbers.extend(values);
        }
        let header = Self {
            variable_name: block.variable_name,
            shape: block.shape,
            dimension_names: block.dimension_names,
            origin: block.origin,
            attributes: Arc::unwrap_or_clone(block.attributes),
            flipped_dims: block.flipped_dims,
            coordinates,
        };
        (header, block.values, numbers)
    }

    /// The block [`split`](Self::split) took apart, checked against its parts.
    pub fn join(self, values: Arc<[f32]>, numbers: &[f64]) -> Result<OctantBlock, String> {
        let volume = self
            .shape
            .iter()
            .try_fold(1usize, |acc, &d| acc.checked_mul(d))
            .ok_or("Block shape overflows")?;
        if volume != values.len() {
            return Err(format!(
                "Block of {} values for shape {:?}",
                values.len(),
                self.shape
            ));
        }
        let [min, max, rest @ ..] = numbers else {
            return Err("Block without its value range".into());
        };
        let mut rest = rest;
        let mut coordinates = HashMap::with_capacity(self.coordinates.len());
        for (name, len) in self.coordinates {
            if len > rest.len() {
                return Err(format!("Coordinate '{name}' runs past the block's numbers"));
            }
            let (values, after) = rest.split_at(len);
            coordinates.insert(name, values.to_vec());
            rest = after;
        }
        if !rest.is_empty() {
            return Err("Block numbers left over".into());
        }
        Ok(OctantBlock {
            strides: OctantBlock::row_major_strides(&self.shape),
            variable_name: self.variable_name,
            shape: self.shape,
            dimension_names: self.dimension_names,
            origin: self.origin,
            values,
            coordinates,
            attributes: Arc::new(self.attributes),
            min_value: *min as f32,
            max_value: *max as f32,
            flipped_dims: self.flipped_dims,
        })
    }
}
