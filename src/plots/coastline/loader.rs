//! Coastline data loading and async prefetching.

use bytemuck::cast_slice;
use std::sync::mpsc::Sender;

use super::types::{Aligned4, CoastlineBuffer, CoastlineLod};

const REPO_URL: &str = "https://raw.githubusercontent.com/lazarusA/octant/main/assets/coastlines";

/// 110m coastline vertices statically embedded at compile time (41 KB).
pub fn coastline_110m_static() -> &'static [f32] {
    static ALIGNED: &Aligned4<[u8]> = &Aligned4(*include_bytes!(
        "../../../assets/coastlines/coastline_110m.bin"
    ));
    cast_slice(&ALIGNED.0)
}

/// Loads a coastline LOD synchronously from local cache/assets, falling back to embedded 110m.
pub fn load_coastline_sync(lod: CoastlineLod) -> CoastlineBuffer {
    if lod == CoastlineLod::Lod110m {
        return CoastlineBuffer::Static(coastline_110m_static());
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(data) = read_local_coastline_file(lod) {
        return CoastlineBuffer::Owned(data.into());
    }

    log::warn!("LOD {:?} not cached locally; falling back to 110m", lod);
    CoastlineBuffer::Static(coastline_110m_static())
}

/// Spawns a background thread to fetch a coastline LOD from disk or network,
/// sending the loaded vertices through `tx`.
pub fn fetch_coastline_async(
    lod: CoastlineLod,
    tx: Sender<Result<(CoastlineLod, Vec<f32>), String>>,
) {
    if lod == CoastlineLod::Lod110m {
        let _ = tx.send(Ok((lod, coastline_110m_static().to_vec())));
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::Builder::new()
        .name(format!("octant-coastline-fetch-{:?}", lod))
        .spawn(move || {
            let result = load_or_download_coastline(lod);
            let _ = tx.send(result.map(|data| (lod, data)));
        })
        .ok();

    #[cfg(target_arch = "wasm32")]
    let _ = tx.send(Err(
        "Higher LOD coastlines unavailable on WebAssembly".to_string()
    ));
}

#[cfg(not(target_arch = "wasm32"))]
fn read_local_coastline_file(lod: CoastlineLod) -> Option<Vec<f32>> {
    let filename = lod.filename();
    let candidates = candidate_paths(filename);

    for path in candidates {
        if let Ok(bytes) = std::fs::read(&path)
            && bytes.len() % (std::mem::size_of::<f32>() * 2) == 0
            && !bytes.is_empty()
        {
            return Some(bytemuck::pod_collect_to_vec(&bytes));
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn load_or_download_coastline(lod: CoastlineLod) -> Result<Vec<f32>, String> {
    if let Some(local) = read_local_coastline_file(lod) {
        return Ok(local);
    }

    let cache_dir = coastline_cache_dir().ok_or("Failed to resolve cache directory")?;
    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {e}"))?;

    let filename = lod.filename();
    let url = format!("{REPO_URL}/{filename}");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Client build error: {e}"))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Download request error: {e}"))?
        .error_for_status()
        .map_err(|e| format!("HTTP status error: {e}"))?;

    let bytes = response
        .bytes()
        .map_err(|e| format!("Bytes read error: {e}"))?;
    if bytes.len() < 8 || !bytes.len().is_multiple_of(std::mem::size_of::<f32>() * 2) {
        return Err("Invalid downloaded coastline payload".to_string());
    }

    let cache_path = cache_dir.join(filename);
    let temp_path = cache_path.with_extension("download");
    std::fs::write(&temp_path, &bytes).map_err(|e| format!("Cache write error: {e}"))?;
    std::fs::rename(&temp_path, &cache_path).map_err(|e| format!("Cache rename error: {e}"))?;

    Ok(bytemuck::pod_collect_to_vec(&bytes))
}

#[cfg(not(target_arch = "wasm32"))]
fn candidate_paths(filename: &str) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::with_capacity(3);
    if let Some(cache_dir) = coastline_cache_dir() {
        paths.push(cache_dir.join(filename));
    }
    paths.push(std::path::PathBuf::from("assets/coastlines").join(filename));
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        paths.push(parent.join("assets/coastlines").join(filename));
    }
    paths
}

#[cfg(not(target_arch = "wasm32"))]
fn coastline_cache_dir() -> Option<std::path::PathBuf> {
    if let Ok(root) = std::env::var("XDG_CACHE_HOME")
        && !root.is_empty()
    {
        return Some(std::path::PathBuf::from(root).join("octant/coastlines"));
    }
    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        return Some(std::path::PathBuf::from(home).join("Library/Caches/Octant/coastlines"));
    }
    #[cfg(target_os = "windows")]
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        return Some(std::path::PathBuf::from(local).join("Octant/coastlines"));
    }
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".cache/octant/coastlines"))
}
