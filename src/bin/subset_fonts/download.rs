//! MD5-verified downloads, cached in `target/subset_fonts/<md5>-<name>`.

use super::sources::Remote;
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
        println!("downloading {url}");
        let bytes = self
            .client
            .get(&url)
            .send()?
            .error_for_status()?
            .bytes()?
            .to_vec();
        let actual = md5_hex(&bytes);
        if actual != remote.md5 {
            return Err(format!(
                "{url}: MD5 mismatch (expected {}, got {actual}); review upstream and update sources.rs",
                remote.md5
            )
            .into());
        }
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&path, &bytes)?;
        Ok(bytes)
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
