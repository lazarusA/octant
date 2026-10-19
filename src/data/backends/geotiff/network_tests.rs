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

/// OpenLandMap-soildb stores pH times 10 with a GDAL scale of 0.1; blocks of
/// its coarsest overview (which carries no metadata of its own) hold pH.
#[test]
#[ignore = "reads remote stores"]
fn scaled_geotiff_reads_physical_values() {
    let entry = crate::catalog::SOIL_CATALOG
        .iter()
        .find(|e| e.key == "olm-soildb-ph-120m")
        .expect("pH entry");
    let store = super::store::GeoTiffBlockStore::open(entry.store).expect("open");
    let meta = store.inspect().expect("inspect");
    let coarsest = meta
        .variables
        .iter()
        .filter(|v| v.name.starts_with("overview_"))
        .min_by_key(|v| v.shape.iter().product::<u64>())
        .expect("an overview");
    let shape: Vec<usize> = coarsest.shape.iter().map(|&n| n as usize).collect();
    let request = crate::data::slice_request::SliceRequest::full_range(&coarsest.name, &shape);
    let block = store.fetch_block(&request).expect("fetch");
    let valid: Vec<f32> = block
        .values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .collect();
    let (lo, hi) = valid
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    eprintln!(
        "{}: {} valid values in {lo}..{hi}",
        coarsest.name,
        valid.len()
    );
    assert!(!valid.is_empty());
    assert!(lo >= 2.0 && hi <= 12.0, "pH, not pH times 10: {lo}..{hi}");
}
