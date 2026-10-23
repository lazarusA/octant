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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static DOWNLOADING: Mutex<()> = Mutex::new(());
/// Chosen models downloading or waiting to: the background one gives way.
static FOREGROUND: AtomicUsize = AtomicUsize::new(0);

/// The folder holding the model's files, downloading those missing, with
/// the share of the weights done; `cancelled` stops it.
pub(super) fn ensure(
    size: WhisperSize,
    progress: &dyn Fn(f32),
    cancelled: &dyn Fn() -> bool,
) -> Result<PathBuf, String> {
    let dir = folder(size)?;
    // A model on disk never waits for another one downloading.
    if on_disk(size, &dir) {
        return Ok(dir);
    }
    FOREGROUND.fetch_add(1, Ordering::SeqCst);
    let result = download(size, &dir, progress, cancelled);
    FOREGROUND.fetch_sub(1, Ordering::SeqCst);
    result.map(|()| dir)
}

/// Downloads `size` quietly on its own thread, once per session, so
/// choosing it later starts at once; a chosen model downloading stops it
/// (until the next session), and a failure is only logged.
pub(super) fn prefetch(size: WhisperSize) {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::Relaxed) {
        return;
    }
    std::thread::spawn(move || {
        let chosen = || FOREGROUND.load(Ordering::SeqCst) > 0;
        let result = folder(size).and_then(|dir| {
            if on_disk(size, &dir) {
                return Ok(());
            }
            download(size, &dir, &|_| {}, &chosen)
        });
        if let Err(e) = result {
            log::warn!("Whisper {} in the background: {e}", size.name());
        }
    });
}

fn folder(size: WhisperSize) -> Result<PathBuf, String> {
    Ok(eframe::storage_dir("Octant")
        .ok_or("no data folder")?
        .join("models/whisper")
        .join(size.name()))
}

fn on_disk(size: WhisperSize, dir: &Path) -> bool {
    models::files(size)
        .iter()
        .all(|(f, _)| dir.join(f).exists())
}

/// Downloads the files of `size` missing from `dir`, one download at a time.
fn download(
    size: WhisperSize,
    dir: &Path,
    progress: &dyn Fn(f32),
    cancelled: &dyn Fn() -> bool,
) -> Result<(), String> {
    let _one_at_a_time = DOWNLOADING.lock().unwrap_or_else(|p| p.into_inner());
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
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
    Ok(())
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
    let got = models::hex(&hasher.finalize());
    if got == sha256 {
        Ok(())
    } else {
        Err(format!("checksum {got}, expected {sha256}"))
    }
}

#[cfg(test)]
mod tests {
    use super::write_verified;
    use sha2::{Digest, Sha256};
    use std::io::Cursor;

    #[test]
    fn only_a_matching_download_is_accepted() {
        let dir = std::env::temp_dir().join(format!("octant-verify-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let part = dir.join("model.part");
        let body = b"weights".to_vec();
        let sha = super::models::hex(&Sha256::digest(&body));
        let (mut seen, never) = (0, &|| false);
        let ok = write_verified(&mut Cursor::new(&body), &part, &sha, never, &mut |n| {
            seen += n
        });
        assert_eq!((ok, seen), (Ok(()), body.len()));
        assert_eq!(std::fs::read(&part).expect("written"), body);
        let wrong = "0".repeat(64);
        let err = write_verified(&mut Cursor::new(&body), &part, &wrong, never, &mut |_| {});
        assert!(err.is_err_and(|e| e.contains("checksum")));
        let stopped = write_verified(&mut Cursor::new(&body), &part, &sha, &|| true, &mut |_| {});
        assert!(stopped.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
