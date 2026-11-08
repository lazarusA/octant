//! The browser's NetCDF store. `hdf5-pure` reads synchronously, which only a Web Worker
//! can: a data worker opens the file (`open`, from `data::worker::host`) and keeps it
//! until its dataset closes (`release`); the page asks a worker for the inspection
//! (`data::worker::inspect`) and for blocks. On the page the store reads nothing.

use std::cell::RefCell;
use std::collections::HashMap;

use hdf5_pure::FormatError;

use super::open::{Shared, open_source};
use super::pure::read_ahead::ReadAhead;
use super::reader::NcReader;
use crate::data::DataSource;
use crate::data::blocks::BlockStoreError;

thread_local! {
    /// The files open in this worker, by URL.
    /// Held as `dyn NcReader` so that only the worker's `open` names the readers, which
    /// keeps `hdf5-pure` out of the page's `.wasm`.
    static OPEN: RefCell<HashMap<String, Shared>> = RefCell::default();
    /// Local files the page handed over, by URL, read by `read_local`.
    static FILES: RefCell<HashMap<String, web_sys::File>> = RefCell::default();
}

#[derive(Debug, Clone)]
pub struct NetCdfBlockStore {
    uri: String,
}

impl NetCdfBlockStore {
    pub fn open_local(uri: &str) -> Result<Self, BlockStoreError> {
        Ok(Self {
            uri: uri.trim_end_matches('/').to_owned(),
        })
    }

    pub fn file_path(&self) -> &str {
        &self.uri
    }

    /// Runs `f` on the file this worker opened.
    pub(super) fn with_reader<R>(
        &self,
        f: impl FnOnce(&dyn NcReader) -> Result<R, BlockStoreError>,
    ) -> Result<R, BlockStoreError> {
        let reader = OPEN.with_borrow(|o| o.get(&self.uri).cloned());
        match reader {
            Some(reader) => f(reader.as_ref()),
            None => Err("NetCDF files are read by the data workers, which are not running".into()),
        }
    }
}

/// Opens `source` in this worker, once (`data::worker::host`).
pub async fn open(source: &DataSource) -> Result<(), String> {
    let uri = source.uri.trim_end_matches('/').to_owned();
    if OPEN.with_borrow(|o| o.contains_key(&uri)) {
        return Ok(());
    }
    let opened = if uri.starts_with(crate::data::backends::local::SCHEME) {
        let file = crate::data::worker::file_from_page(&uri).await?;
        let len = file.size() as u64;
        FILES.with_borrow_mut(|f| f.insert(uri.clone(), file));
        let local = uri.clone();
        open_source(
            ReadAhead::new(len, move |o, n| read_local(&local, o, n)),
            &uri,
        )
    } else {
        let len = crate::data::backends::http::fetch_url_size(&uri).await?;
        let url = uri.clone();
        let source = ReadAhead::new(len, move |o, n| read_http(&url, len, o, n));
        open_source(source, &uri)
    };
    let reader = opened.map_err(|e| e.to_string())?;
    OPEN.with_borrow_mut(|o| o.insert(uri, reader));
    Ok(())
}

/// Lets go of the file at `uri`.
pub fn release(uri: &str) {
    let uri = uri.trim_end_matches('/');
    OPEN.with_borrow_mut(|o| o.remove(uri));
    FILES.with_borrow_mut(|f| f.remove(uri));
}

fn source_error(what: &str, e: impl std::fmt::Debug) -> FormatError {
    FormatError::Source(format!("{what}: {e:?}"))
}

/// `length` bytes at `offset` of the local file at `uri`, read by `FileReaderSync` from the
/// `File` in [`FILES`].
fn read_local(uri: &str, offset: u64, length: u64) -> Result<Vec<u8>, FormatError> {
    let end = offset.saturating_add(length);
    let slice = FILES.with_borrow(|f| {
        let file = f.get(uri).ok_or_else(|| source_error(uri, "released"))?;
        (file.slice_with_f64_and_f64(offset as f64, end as f64)).map_err(|e| source_error(uri, e))
    })?;
    let reader = web_sys::FileReaderSync::new().map_err(|e| source_error(uri, e))?;
    let bytes = (reader.read_as_array_buffer(&slice)).map_err(|e| source_error(uri, e))?;
    let mut buf = vec![0; length as usize];
    copy(&js_sys::Uint8Array::new(&bytes), &mut buf, uri)?;
    Ok(buf)
}

/// `length` bytes at `offset` of the remote file at `url`, `len` bytes long, by a
/// synchronous ranged request (the server must allow `Range`).
fn read_http(url: &str, len: u64, offset: u64, length: u64) -> Result<Vec<u8>, FormatError> {
    let mut buf = vec![0; length as usize];
    if buf.is_empty() {
        return Ok(buf);
    }
    let last = offset + length - 1;
    let fail = |e: wasm_bindgen::JsValue| source_error(url, e);
    let xhr = web_sys::XmlHttpRequest::new().map_err(fail)?;
    xhr.open_with_async("GET", url, false).map_err(fail)?;
    xhr.set_request_header("Range", &format!("bytes={offset}-{last}"))
        .map_err(fail)?;
    xhr.set_response_type(web_sys::XmlHttpRequestResponseType::Arraybuffer);
    // As `http::fetch` gives up: a stalled server would hold this worker for good.
    xhr.set_timeout(60_000);
    xhr.send().map_err(fail)?;
    let bytes = js_sys::Uint8Array::new(&xhr.response().map_err(fail)?);
    match xhr.status().map_err(fail)? {
        206 => copy(&bytes, &mut buf, url)?,
        // The whole file asked for. Otherwise a server ignoring `Range` would send the
        // whole file for every block read.
        200 if offset == 0 && length == len => copy(&bytes, &mut buf, url)?,
        200 => {
            let why = "the server does not support range requests";
            return Err(source_error(url, why));
        }
        status => return Err(source_error(url, format!("HTTP {status}"))),
    }
    Ok(buf)
}

fn copy(bytes: &js_sys::Uint8Array, buf: &mut [u8], what: &str) -> Result<(), FormatError> {
    if bytes.length() as usize != buf.len() {
        return Err(source_error(what, "short read"));
    }
    bytes.copy_to(buf);
    Ok(())
}
