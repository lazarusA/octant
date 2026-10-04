//! Downloads (with a per-run memory cache and an on-disk cache for large
//! archives), verifies and converts every source of a family.

use crate::families::{Family, License, MIT_TEMPLATE, Remote};
use crate::sources;
use md5::{Digest, Md5};
use octant::utils::colormap::format::MapRecord;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::PathBuf;

pub type Error = Box<dyn std::error::Error>;

pub struct Fetcher {
    client: reqwest::blocking::Client,
    cache: HashMap<String, String>,
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

/// Errors unless `bytes` (downloaded from `url`) have the `expected` MD5.
fn verify(url: &str, expected: &str, bytes: &[u8]) -> Result<(), String> {
    let actual = md5_hex(bytes);
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{url}: MD5 mismatch (expected {expected}, got {actual}); \
             upstream changed: review it and update the checksum in families/"
        ))
    }
}

/// Whether `url` names a commit (a 40-hex-digit path segment), so its content is fixed.
fn is_commit_pinned(url: &str) -> bool {
    url.split(['/', '?'])
        .any(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}

impl Fetcher {
    pub fn new() -> Result<Self, Error> {
        // GitHub's API rejects requests without a User-Agent.
        let client = reqwest::blocking::Client::builder()
            .user_agent("octant-fetch-colormaps")
            .timeout(std::time::Duration::from_secs(600))
            .build()?;
        Ok(Self {
            client,
            cache: HashMap::new(),
        })
    }

    fn download(&self, url: &str) -> Result<Vec<u8>, Error> {
        let response = self.client.get(url).send()?.error_for_status()?;
        Ok(response.bytes()?.to_vec())
    }

    /// A text file whose content must match its MD5.
    pub fn text(&mut self, remote: &Remote) -> Result<&str, Error> {
        self.load(remote.url, Some(remote.md5))
    }

    /// A text file derived from a commit-pinned base URL (files listed upstream).
    pub fn pinned_text(&mut self, url: &str) -> Result<&str, Error> {
        if !is_commit_pinned(url) {
            return Err(format!("{url}: neither pinned to a commit nor checksummed").into());
        }
        self.load(url, None)
    }

    fn load(&mut self, url: &str, md5: Option<&str>) -> Result<&str, Error> {
        if !self.cache.contains_key(url) {
            let bytes = self.download(url)?;
            if let Some(expected) = md5 {
                verify(url, expected, &bytes)?;
            }
            let text = String::from_utf8(bytes).map_err(|e| format!("{url}: {e}"))?;
            self.cache.insert(url.to_string(), text);
        }
        Ok(self.cache.get(url).map(String::as_str).unwrap_or_default())
    }

    /// Large downloads are kept in `target/fetch_colormaps/<md5>-<name>` between
    /// runs. A cached file with another checksum is downloaded again once.
    pub fn cached_bytes(&mut self, remote: &Remote, name: &str) -> Result<Vec<u8>, Error> {
        let dir =
            PathBuf::from(std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()))
                .join("fetch_colormaps");
        let path = dir.join(format!("{}-{name}", remote.md5));
        if let Ok(bytes) = std::fs::read(&path) {
            if md5_hex(&bytes) == remote.md5 {
                return Ok(bytes);
            }
            println!("  {} is corrupt, downloading again", path.display());
        }
        println!("  downloading {} ...", remote.url);
        let bytes = self.download(remote.url)?;
        verify(remote.url, remote.md5, &bytes)?;
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&path, &bytes)?;
        Ok(bytes)
    }

    /// License text(s) of a family, with its attribution notice first.
    pub fn license_text(&mut self, family: &Family) -> Result<String, Error> {
        let text = match family.license {
            License::Urls(remotes) => {
                let mut texts = Vec::with_capacity(remotes.len());
                for remote in remotes {
                    let text = self.text(remote)?.trim_end();
                    texts.push(format!("[{}]\n\n{text}", remote.url));
                }
                texts.join("\n\n----------------------------------------\n\n")
            }
            License::Inline(text) => text.to_string(),
            License::Mit(copyright) => MIT_TEMPLATE.replace("{copyright}", copyright),
        };
        Ok(match family.notice {
            Some(notice) => format!("{notice}\n\n{text}"),
            None => text,
        })
    }

    /// Converts every map of a family. Each map needs at least two colors and a
    /// name unique within the family; a family without maps is an error.
    pub fn family_maps(
        &mut self,
        family: &Family,
        family_idx: u8,
    ) -> Result<Vec<MapRecord>, Error> {
        let mut seen = HashSet::new();
        let mut maps = Vec::new();
        for source in family.sources {
            for (name, kind, stops) in sources::convert(self, source)? {
                if stops.len() < 2 {
                    return Err(
                        format!("{}: `{name}` has {} colors", family.name, stops.len()).into(),
                    );
                }
                if !seen.insert(name.clone()) {
                    return Err(format!("{}: duplicate map name `{name}`", family.name).into());
                }
                maps.push(MapRecord {
                    name,
                    family: family_idx,
                    kind,
                    stops,
                });
            }
        }
        if maps.is_empty() {
            return Err(format!("{}: no maps converted", family.name).into());
        }
        Ok(maps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_and_verification() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert!(verify("u", "d41d8cd98f00b204e9800998ecf8427e", b"").is_ok());
        let Err(err) = verify("u", "00000000000000000000000000000000", b"") else {
            panic!("mismatch accepted")
        };
        assert!(err.contains("expected 0000") && err.contains("got d41d8"));
    }

    #[test]
    fn commit_pinned_urls() {
        assert!(is_commit_pinned(
            "https://raw.githubusercontent.com/a/b/0123456789abcdef0123456789abcdef01234567/x.py"
        ));
        assert!(!is_commit_pinned(
            "https://raw.githubusercontent.com/a/b/master/x.py"
        ));
    }
}
