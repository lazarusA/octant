//! Downloads (with a per-run memory cache and an on-disk cache for large
//! archives) and converts every source of a family.

use crate::families::{Family, License, MIT_TEMPLATE};
use crate::sources::{self, to_rgb8};
use octant::utils::colormap::format::MapRecord;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub type Error = Box<dyn std::error::Error>;

pub struct Fetcher {
    client: reqwest::blocking::Client,
    cache: HashMap<String, String>,
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

    fn get(&self, url: &str) -> Result<reqwest::blocking::Response, Error> {
        Ok(self.client.get(url).send()?.error_for_status()?)
    }

    pub fn text(&mut self, url: &str) -> Result<&str, Error> {
        if !self.cache.contains_key(url) {
            let text = self.get(url)?.text()?;
            self.cache.insert(url.to_string(), text);
        }
        Ok(self.cache.get(url).map(String::as_str).unwrap_or_default())
    }

    /// Large downloads are kept in `target/fetch_colormaps/<name>` between runs.
    pub fn cached_bytes(&mut self, url: &str, name: &str) -> Result<Vec<u8>, Error> {
        let dir =
            PathBuf::from(std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()))
                .join("fetch_colormaps");
        let path = dir.join(name);
        if let Ok(bytes) = std::fs::read(&path) {
            return Ok(bytes);
        }
        println!("  downloading {url} ...");
        let bytes = self.get(url)?.bytes()?.to_vec();
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&path, &bytes)?;
        Ok(bytes)
    }

    /// License text(s) of a family, with its attribution notice first.
    pub fn license_text(&mut self, family: &Family) -> Result<String, Error> {
        let text = match family.license {
            License::Urls(urls) => {
                let mut texts = Vec::with_capacity(urls.len());
                for url in urls {
                    texts.push(format!("[{url}]\n\n{}", self.text(url)?.trim_end()));
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

    /// Converts every map of a family, deduplicated by name.
    pub fn family_maps(
        &mut self,
        family: &Family,
        family_idx: u8,
    ) -> Result<Vec<MapRecord>, Error> {
        let mut seen = HashSet::new();
        let mut maps = Vec::new();
        for source in family.sources {
            for (name, kind, colors) in sources::convert(self, source)? {
                let stops = to_rgb8(&colors);
                if stops.len() < 2 {
                    return Err(
                        format!("{}: `{name}` has {} colors", family.name, stops.len()).into(),
                    );
                }
                if seen.insert(name.clone()) {
                    maps.push(MapRecord {
                        name,
                        family: family_idx,
                        kind,
                        stops,
                    });
                }
            }
        }
        Ok(maps)
    }
}
