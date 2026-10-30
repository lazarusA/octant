//! Opens every Microscopy catalog entry over the network, through the
//! desktop path: run by hand with
//! `cargo test --lib ome_catalog_entries_open -- --ignored --nocapture`.

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "reads remote stores"]
fn ome_catalog_entries_open() {
    use crate::data::blocks::BlockStore;
    let mut failures = Vec::new();
    for entry in crate::catalog::MICROSCOPY_CATALOG {
        let opened = crate::data::backends::zarr::ZarrBlockStore::open_remote(entry.store)
            .map_err(|e| e.to_string())
            .and_then(|store| store.inspect().map_err(|e| e.to_string()));
        match opened {
            Ok(meta) => {
                let ome = meta.ome.as_deref();
                let plate = ome.and_then(|o| o.plate.as_ref()).map(|p| p.wells.len());
                let first = meta.variables.first();
                eprintln!(
                    "{}: {} variables {:?}, plate wells {:?}, version {:?}, units {:?}",
                    entry.key,
                    meta.variables.len(),
                    meta.variables
                        .iter()
                        .take(4)
                        .map(|v| v.name.as_str())
                        .collect::<Vec<_>>(),
                    plate,
                    ome.map(|o| o.version.as_str()),
                    first.map(|v| {
                        v.dimension_names
                            .iter()
                            .map(|d| v.dimension_unit(d).unwrap_or("-"))
                            .collect::<Vec<_>>()
                    }),
                );
                if let Ok(storage) =
                    crate::data::backends::zarr::storage::build_sync_store(entry.store)
                    && let Some(found) =
                        super::discover_direct(storage, &super::MetaCache::shared())
                {
                    for w in &found.warnings {
                        eprintln!("  warning: {w}");
                    }
                }
                if meta.variables.is_empty() && plate.is_none() {
                    failures.push(format!("{}: nothing found", entry.key));
                }
                if meta.variables.iter().any(|v| v.ome.is_none()) {
                    failures.push(format!("{}: variables without OME metadata", entry.key));
                }
            }
            Err(e) => failures.push(format!("{}: {e}", entry.key)),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
