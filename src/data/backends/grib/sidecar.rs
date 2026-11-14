//! The sidecar index of a remote GRIB file, which lists where its messages start: NOAA's
//! wgrib2 inventory beside the file (`.idx`), or ECMWF's in place of its extension
//! (`.index`). Without one, the file's messages are found one after another.

/// The message offsets the sidecar of the file at `url`, `len` bytes long, lists, sorted
/// and each once (submessages share their message's); `None` when it has none.
pub fn sidecar_offsets(url: &str, len: u64) -> Option<Vec<u64>> {
    let runtime = crate::utils::executor::get_shared_tokio_rt();
    let mut candidates = vec![format!("{url}.idx")];
    if let Some(stem) = url.strip_suffix(".grib2") {
        candidates.push(format!("{stem}.index"));
    }
    candidates.into_iter().find_map(|candidate| {
        let fetched = crate::data::backends::http::fetch_url_bytes(&candidate);
        let text = String::from_utf8(runtime.block_on(fetched).ok()?).ok()?;
        let entries = gribberish::index::parse_index(&text, Some(len)).ok()?;
        let mut offsets: Vec<u64> = entries.iter().map(|e| e.offset).collect();
        offsets.sort_unstable();
        offsets.dedup();
        (!offsets.is_empty()).then_some(offsets)
    })
}
