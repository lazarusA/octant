//! Whisper's files from OpenAI's Hugging Face repositories, pinned to a
//! commit and checked by SHA-256, downloaded once into the app's data folder
//! (`<storage>/models/whisper/<name>/`). A file is written as `.part` and
//! renamed only once its hash matches, so a file present is a whole one.
//! One download runs at a time, so a background one and a chosen one never
//! write the same file.

use crate::voice::stt::WhisperSize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static DOWNLOADING: Mutex<()> = Mutex::new(());

struct Spec {
    repo: &'static str,
    revision: &'static str,
    config: &'static str,
    tokenizer: &'static str,
    weights: &'static str,
    weights_bytes: u64,
}

/// `tokenizer.json` of every multilingual model before large-v3.
const TOKENIZER: &str = "27fc476bfe7f17299480be2273fc0608e4d5a99aba2ab5dec5374b4482d1a566";

fn spec(size: WhisperSize) -> Spec {
    match size {
        WhisperSize::Tiny => Spec {
            repo: "openai/whisper-tiny",
            revision: "169d4a4341b33bc18d8881c4b69c2e104e1cc0af",
            config: "ffdccec4f3211f4c63310f2b7098f309fe70f3952cedc5e4d11e43f5b2379b98",
            tokenizer: TOKENIZER,
            weights: "7ebd0e69e78190ffe1438491fa05cc1f5c1aa3a4c4db3bc1723adbb551ea2395",
            weights_bytes: 151_061_672,
        },
        WhisperSize::Base => Spec {
            repo: "openai/whisper-base",
            revision: "e37978b90ca9030d5170a5c07aadb050351a65bb",
            config: "a153c53883a6799b6f056b4a8d1a515c9926d03994682ba88a7616618d7da0c1",
            tokenizer: TOKENIZER,
            weights: "07cadb9f25677c8d50df603e66a98fbd842cce45047139baeb16e6219a1e807b",
            weights_bytes: 290_403_936,
        },
        WhisperSize::Small => Spec {
            repo: "openai/whisper-small",
            revision: "973afd24965f72e36ca33b3055d56a652f456b4d",
            config: "e6a2b489da1b5aed65a8eb8d1e7466fa867ad5643a8bc138ba708bd56b2875c4",
            tokenizer: TOKENIZER,
            weights: "1d7734884874f1a1513ed9aa760a4f8e97aaa02fd6d93a3a85d27b2ae9ca596b",
            weights_bytes: 966_995_080,
        },
        WhisperSize::Turbo => Spec {
            repo: "openai/whisper-large-v3-turbo",
            revision: "41f01f3fe87f28c78e2fbf8b568835947dd65ed9",
            config: "c5b526b3e3cd64cd8940dabb45e8ba726629e22d8ed389c29b552f9140daf04a",
            tokenizer: "297b13372ac43916285644fb9687add3cc62ee2a1adb60da3dc25cc94c1871fd",
            weights: "542566a422ae4f3fd23f1ba11add198fca01bbf82e66e6a2857b3f608b1eb9d1",
            weights_bytes: 1_617_824_864,
        },
    }
}

/// Megabytes the model's weights take on disk.
pub fn megabytes(size: WhisperSize) -> u64 {
    spec(size).weights_bytes / 1_000_000
}

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
    let s = spec(size);
    let total = s.weights_bytes as f32;
    for (file, sha) in [
        ("config.json", s.config),
        ("tokenizer.json", s.tokenizer),
        ("model.safetensors", s.weights),
    ] {
        let dest = dir.join(file);
        if dest.exists() {
            continue;
        }
        let url = format!(
            "https://huggingface.co/{}/resolve/{}/{file}",
            s.repo, s.revision
        );
        let mut done = 0u64;
        fetch(&url, &dest, sha, cancelled, |n| {
            done += n as u64;
            if file == "model.safetensors" {
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
    let result = (|| {
        let mut file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
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
        if got != sha256 {
            return Err(format!("{url}: checksum {got}, expected {sha256}"));
        }
        std::fs::rename(&part, dest).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result
}
