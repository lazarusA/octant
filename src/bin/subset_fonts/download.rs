//! MD5-verified downloads, cached in `target/subset_fonts/<md5>-<name>`.

use super::pins::Remote;
use md5::{Digest, Md5};
use std::fmt::Write as _;
use std::path::PathBuf;

pub type Error = Box<dyn std::error::Error>;

pub struct Downloader {
    client: reqwest::blocking::Client,
}

impl Downloader {
    pub fn new() -> Result<Self, Error> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("octant-subset-fonts")
            .timeout(std::time::Duration::from_secs(600))
            .build()?;
        Ok(Self { client })
    }

    /// `remote`'s bytes, from the cache when its checksum still matches.
    pub fn cached(&self, remote: &Remote, name: &str) -> Result<Vec<u8>, Error> {
        let dir =
            PathBuf::from(std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()))
                .join("subset_fonts");
        let path = dir.join(format!("{}-{name}", remote.md5));
        if let Ok(bytes) = std::fs::read(&path)
            && md5_hex(&bytes) == remote.md5
        {
            return Ok(bytes);
        }
        let url = remote.url();
        let bytes = self.download(&url)?;
        let actual = md5_hex(&bytes);
        if actual != remote.md5 {
            return Err(format!(
                "{url}: MD5 mismatch (expected {}, got {actual}); review upstream and update pins.rs",
                remote.md5
            )
            .into());
        }
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&path, &bytes)?;
        Ok(bytes)
    }

    /// `url`'s bytes, retrying transient failures (network errors and
    /// server errors) a few times with growing pauses.
    fn download(&self, url: &str) -> Result<Vec<u8>, Error> {
        const ATTEMPTS: u32 = 4;
        let mut attempt = 1;
        loop {
            println!("downloading {url}");
            let result = self
                .client
                .get(url)
                .send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .and_then(reqwest::blocking::Response::bytes);
            match result {
                Ok(bytes) => return Ok(bytes.to_vec()),
                Err(e)
                    if attempt < ATTEMPTS && !e.status().is_some_and(|s| s.is_client_error()) =>
                {
                    eprintln!("{url}: {e}; retrying");
                    std::thread::sleep(std::time::Duration::from_secs(2u64.pow(attempt)));
                    attempt += 1;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// Lowercase hex MD5 of `bytes`.
fn md5_hex(bytes: &[u8]) -> String {
    Md5::digest(bytes)
        .iter()
        .fold(String::with_capacity(32), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}
