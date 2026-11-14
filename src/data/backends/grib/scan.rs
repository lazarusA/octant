//! The messages of a GRIB2 file, read from their heads: each message's identity, level,
//! times and grid sit in its first sections, so a 4 KiB read per message describes it
//! without its data. Messages may be padded apart (ECMWF), so the next one is searched for;
//! a sidecar index (`scan_at`) gives them all at once, their heads read in parallel.

use gribberish::message::read_message;
use gribberish::message_metadata::MessageMetadata;

use super::sections::MAX_MESSAGE_BYTES;

use crate::data::backends::netcdf::pure::read_ahead::{BLOCK, RangeSource};
use crate::data::backends::netcdf::reader::failed;
use crate::data::blocks::BlockStoreError;

/// Bytes read at each message; a message whose sections up to its data do not fit is
/// read whole.
const HEAD: u64 = 4096;
/// The indicator section: `GRIB`, two reserved bytes, discipline, edition, total length.
const INDICATOR: u64 = 16;

/// One message: where it lies and what it holds.
pub struct Field {
    pub offset: u64,
    pub len: u64,
    pub meta: MessageMetadata,
}

/// What the scan found: the messages read, and the offsets of those it could not describe.
#[derive(Default)]
pub struct Scan {
    pub fields: Vec<Field>,
    pub unreadable: Vec<u64>,
}

impl Scan {
    fn push(&mut self, offset: u64, len: u64, meta: Option<MessageMetadata>) {
        match meta {
            Some(meta) => self.fields.push(Field { offset, len, meta }),
            None => self.unreadable.push(offset),
        }
    }
}

/// The bytes [`is_grib`] looks at.
pub const SIGNATURE_BYTES: usize = 64;

/// Whether `head`, a file's first [`SIGNATURE_BYTES`], is a GRIB file's: `GRIB` within
/// them (a WMO bulletin header may come first, which the scan skips), and no NetCDF, HDF5
/// or MATLAB signature, whose headers may name GRIB.
pub fn is_grib(head: &[u8]) -> bool {
    let others: [&[u8]; 3] = [b"CDF", b"\x89HDF", b"MATLAB"];
    !others.iter().any(|s| head.starts_with(s)) && head.windows(4).any(|w| w == b"GRIB")
}

/// The metadata of the message `bytes` start with, `None` when its sections are cut short,
/// shorter than their templates (`sections::fit_templates`), its grid has more points than
/// Octant decodes (`MAX_POINTS`: opening computes its rows' and columns' coordinates), or it
/// describes something gribberish does not read.
fn metadata(bytes: &[u8]) -> Option<MessageMetadata> {
    if !super::sections::fit_templates(bytes) {
        return None;
    }
    let meta = MessageMetadata::try_from(&read_message(bytes, 0)?).ok()?;
    let (ny, nx) = meta.grid_shape;
    let points = ny.checked_mul(nx).and_then(|n| u64::try_from(n).ok());
    points
        .is_some_and(|n| n <= super::sections::MAX_POINTS)
        .then_some(meta)
}

/// `len` bytes at `offset`: through the read-ahead when `near` the previous read, else
/// straight from the file (heads a block or more apart share no read-ahead block).
fn read(
    source: &dyn RangeSource,
    (offset, len): (u64, u64),
    near: bool,
    name: &str,
) -> Result<Vec<u8>, BlockStoreError> {
    let size = usize::try_from(len).map_err(|_| format!("'{name}': message too large"))?;
    let mut bytes = vec![0; size];
    let bytes_read = match near {
        true => source.read_at(offset, &mut bytes),
        false => source.read_direct(offset, &mut bytes),
    };
    bytes_read.map_err(|e| failed(&format!("Failed to read '{name}'"), e))?;
    Ok(bytes)
}

/// The head read at `offset`: up to `HEAD` bytes, fewer at the end of the file.
fn head_at(
    source: &dyn RangeSource,
    offset: u64,
    near: bool,
    name: &str,
) -> Result<Vec<u8>, BlockStoreError> {
    read(
        source,
        (offset, HEAD.min(source.len().saturating_sub(offset))),
        near,
        name,
    )
}

/// The length and metadata (`None`: unreadable) of the message at `offset`, whose first
/// bytes are `head`; read whole when its head does not describe it, unless it is longer than
/// `MAX_MESSAGE_BYTES` (unreadable, never reserved).
fn describe(
    source: &dyn RangeSource,
    offset: u64,
    head: &[u8],
    name: &str,
) -> Result<(u64, Option<MessageMetadata>), BlockStoreError> {
    let len = message_length(head, name)?;
    if len < INDICATOR || len > source.len().saturating_sub(offset) {
        return Err(format!("'{name}': message at byte {offset} is cut short").into());
    }
    let head = &head[..usize::try_from(len).map_or(head.len(), |len| head.len().min(len))];
    let meta = match metadata(head) {
        Some(meta) => Some(meta),
        None if len > head.len() as u64 && len <= MAX_MESSAGE_BYTES => {
            metadata(&read(source, (offset, len), false, name)?)
        }
        None => None,
    };
    Ok((len, meta))
}

/// Every message of the file `source` reads, one after another, `name` naming it in errors.
pub fn scan(source: &dyn RangeSource, name: &str) -> Result<Scan, BlockStoreError> {
    let mut scan = Scan::default();
    let (mut offset, mut near) = (0, true);
    while source.len().saturating_sub(offset) >= INDICATOR {
        let head = head_at(source, offset, near, name)?;
        let Some(at) = head.windows(4).position(|w| w == b"GRIB") else {
            offset += head.len() as u64 - 3;
            continue;
        };
        if at > 0 {
            offset += at as u64;
            continue;
        }
        let (len, meta) = describe(source, offset, &head, name)?;
        scan.push(offset, len, meta);
        (offset, near) = (offset + len, len < BLOCK);
    }
    Ok(scan)
}

/// The messages at `offsets` (a sidecar index's, ascending), their heads read `THREADS` at
/// a time: remote reads wait on the network, not the processor.
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_at(
    source: &(dyn RangeSource + Sync),
    offsets: &[u64],
    name: &str,
) -> Result<Scan, BlockStoreError> {
    const THREADS: usize = 16;
    let one = |offset: u64| {
        let head = head_at(source, offset, false, name)?;
        if !head.starts_with(b"GRIB") {
            return Err(format!(
                "'{name}': its index points at byte {offset}, where no message starts"
            )
            .into());
        }
        describe(source, offset, &head, name).map(|(len, meta)| (offset, len, meta))
    };
    let per_thread = offsets.len().div_ceil(THREADS).max(1);
    let described = std::thread::scope(|s| {
        let threads: Vec<_> = (offsets.chunks(per_thread))
            .map(|part| s.spawn(move || part.iter().map(|&o| one(o)).collect::<Vec<_>>()))
            .collect();
        (threads.into_iter())
            .map(|t| {
                t.join()
                    .unwrap_or_else(|_| vec![Err("a GRIB scan thread failed".into())])
            })
            .collect::<Vec<_>>()
    });
    let mut scan = Scan::default();
    for result in described.into_iter().flatten() {
        let (offset, len, meta) = result?;
        scan.push(offset, len, meta);
    }
    Ok(scan)
}

/// The total length of the GRIB2 message `head` starts with (its indicator section's).
fn message_length(head: &[u8], name: &str) -> Result<u64, BlockStoreError> {
    match head.get(7) {
        Some(2) => {}
        Some(1) => return Err(format!("'{name}' is GRIB1, which is not supported yet").into()),
        _ => return Err(format!("'{name}' holds a GRIB edition Octant does not read").into()),
    }
    let message =
        read_message(head, 0).ok_or_else(|| format!("'{name}': message header cut short"))?;
    Ok(message.len() as u64)
}
