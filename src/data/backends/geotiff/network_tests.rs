//! Opens every GeoTIFF and Soil catalog entry over the network, through the
//! desktop path, and checks that the browser can read exactly the entries not
//! marked desktop-only (their servers answer a cross-origin request): run by
//! hand with `cargo test --lib geotiff_catalog_entries_open -- --ignored --nocapture`.

use crate::catalog::{CatalogCategoryFilter, get_catalog_entries};
use crate::data::blocks::BlockStore;

/// Whether `url` answers a cross-origin range request with an
/// `Access-Control-Allow-Origin` header.
fn allows_browser(client: &reqwest::blocking::Client, url: &str) -> Result<bool, String> {
    let response = client
        .get(url)
        .header("Origin", "https://example.org")
        .header("Range", "bytes=0-1023")
        .send()
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    Ok(response
        .headers()
        .contains_key(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN))
}

#[test]
#[ignore = "reads remote stores"]
fn geotiff_catalog_entries_open() {
    let client = reqwest::blocking::Client::new();
    let mut failures = Vec::new();
    let categories = [CatalogCategoryFilter::GeoTiff, CatalogCategoryFilter::Soil];
    for entry in categories.into_iter().flat_map(get_catalog_entries) {
        match allows_browser(&client, entry.store) {
            Ok(cors) if cors == entry.desktop_only() => failures.push(format!(
                "{}: CORS {cors}, but desktop_only is {}",
                entry.key,
                entry.desktop_only()
            )),
            Ok(_) => {}
            Err(e) => failures.push(format!("{}: {e}", entry.key)),
        }
        let opened = super::store::GeoTiffBlockStore::open(entry.store)
            .map_err(|e| e.to_string())
            .and_then(|store| store.inspect().map_err(|e| e.to_string()));
        match opened {
            Ok(meta) => {
                eprintln!(
                    "{}: {} variables {:?}",
                    entry.key,
                    meta.variables.len(),
                    meta.variables
                        .iter()
                        .take(4)
                        .map(|v| (v.name.as_str(), v.shape.as_slice()))
                        .collect::<Vec<_>>(),
                );
                if meta.variables.is_empty() {
                    failures.push(format!("{}: no variables", entry.key));
                }
            }
            Err(e) => failures.push(format!("{}: {e}", entry.key)),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
