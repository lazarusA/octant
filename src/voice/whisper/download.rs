//! Whisper's files (`models.rs`) downloaded once into the app's data folder
//! (`<storage>/models/whisper/<name>/`). A file is written as `.part` and
//! renamed only once its hash matches, so a file present is a whole one.
//! One download runs at a time, so a background one and a chosen one never
//! write the same file.

use super::models::{self, WEIGHTS};
use crate::voice::stt::WhisperSize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static DOWNLOADING: Mutex<()> = Mutex::new(());

/// The folder holding the model's files, downloading those missing, with
/// the share of the weights done; `cancelled` stops it.
pub(super) fn ensure(
    size: WhisperSize,
    progress: &dyn Fn(f32),
    cancelled: &dyn Fn() -> bool,
) -> Result<PathBuf, String> {
    let _one_at_a_time = DOWNLOADING.lock().unwrap_or_else(|p| p.into_inner());
    let dir = eframe::storage_dir("Octant")
        .ok_or("no data folder")?
        .join("models/whisper")
        .join(size.name());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let total = models::weights_bytes(size) as f32;
    for (file, sha) in models::files(size) {
        let dest = dir.join(file);
        if dest.exists() {
            continue;
        }
        let url = models::url(size, file);
        let mut done = 0u64;
        fetch(&url, &dest, sha, cancelled, |n| {
            done += n as u64;
            if file == WEIGHTS {
                progress(done as f32 / total);
            }
        })?;
    }
    Ok(dir)
}

/// Downloads `size` quietly on its own thread, once per session, so
/// choosing it later starts at once; a failure is only logged.
pub(super) fn prefetch(size: WhisperSize) {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::Relaxed) {
        return;
    }
    std::thread::spawn(move || {
        if let Err(e) = ensure(size, &|_| {}, &|| false) {
            log::warn!("Whisper {} in the background: {e}", size.name());
        }
    });
}

fn fetch(
    url: &str,
    dest: &Path,
    sha256: &str,
    cancelled: &dyn Fn() -> bool,
    mut progress: impl FnMut(usize),
) -> Result<(), String> {
    // No overall timeout: the largest model takes minutes on a slow line.
    let client = reqwest::blocking::Client::builder()
        .timeout(None)
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let mut body = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| e.to_string())?;
    let part = dest.with_extension("part");
    let result = write_verified(&mut body, &part, sha256, cancelled, &mut progress)
        .map_err(|e| format!("{url}: {e}"))
        .and_then(|()| std::fs::rename(&part, dest).map_err(|e| e.to_string()));
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}

/// Copies `body` into `part`, checking its SHA-256 against `sha256`.
fn write_verified(
    body: &mut impl Read,
    part: &Path,
    sha256: &str,
    cancelled: &dyn Fn() -> bool,
    progress: &mut impl FnMut(usize),
) -> Result<(), String> {
    let mut file = std::fs::File::create(part).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        if cancelled() {
            return Err("cancelled".to_string());
        }
        let n = body.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        hasher.update(&buf[..n]);
        progress(n);
    }
    let got: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if got == sha256 {
        Ok(())
    } else {
        Err(format!("checksum {got}, expected {sha256}"))
    }
}
