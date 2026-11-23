//! Multiscale fixtures: GeoZarr pyramids of three levels in memory.

use std::sync::Arc;

use serde_json::json;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{StoreKey, WritableStorageTraits};

use super::{TARGET, register_store, write_array_with};
use crate::app::OctantApp;
use crate::data::DatasetMetadata;

fn put_json(store: &MemoryStore, key: &str, value: serde_json::Value) {
    let bytes = serde_json::to_vec(&value).expect("json");
    store
        .set(&StoreKey::new(key).expect("key"), bytes.into())
        .expect("set");
}

/// A GeoZarr pyramid: `t2m` and `sst(time, lat, lon)` at 3x64x64 (level 0),
/// 3x32x32 and 3x16x16, each level a child group of the root's layout.
pub(crate) fn pyramid_app() -> (OctantApp, DatasetMetadata) {
    pyramid_of(64, true, &[TIME])
}

/// `pyramid_app` with `depth(depth, lat, lon)` at 16x64x64, 8x32x32 and
/// 4x16x16: every dimension halved per level, as 3D OME-Zarr volumes are.
pub(crate) fn iso_pyramid_app() -> (OctantApp, DatasetMetadata) {
    pyramid_of(64, true, &[DEPTH])
}

/// `iso_pyramid_app` with `time` first: `(time, depth, lat, lon)` at
/// 3x16x64x64, 3x8x32x32 and 3x4x16x16.
pub(crate) fn time_depth_pyramid_app() -> (OctantApp, DatasetMetadata) {
    pyramid_of(64, true, &[TIME, DEPTH])
}

/// A leading dimension of a pyramid: name, level-0 length, factor per level.
struct Lead(&'static str, u64, u64);
const TIME: Lead = Lead("time", 3, 1);
const DEPTH: Lead = Lead("depth", 16, 2);

/// `pyramid_app` at 3x8192x8192 (level 0, past one GPU storage buffer),
/// 3x4096x4096 and 3x2048x2048, metadata only.
pub(crate) fn big_pyramid_app() -> (OctantApp, DatasetMetadata) {
    pyramid_of(8192, false, &[TIME])
}

/// The pyramid of `pyramid_app` with level 0 `n` x `n` and `leads` first,
/// its values written when `values`.
fn pyramid_of(size: u64, values: bool, leads: &[Lead]) -> (OctantApp, DatasetMetadata) {
    let scale: Vec<f64> = leads.iter().map(|d| d.2 as f64).chain([2.0, 2.0]).collect();
    let store = Arc::new(MemoryStore::new());
    let layout: Vec<_> = (0..3)
        .map(|l| match l {
            0 => json!({ "asset": "0" }),
            _ => json!({
                "asset": l.to_string(), "derived_from": (l - 1).to_string(),
                "transform": { "scale": scale }
            }),
        })
        .collect();
    let group = |attrs| json!({ "zarr_format": 3, "node_type": "group", "attributes": attrs });
    put_json(
        &store,
        "zarr.json",
        group(json!({ "multiscales": { "layout": layout } })),
    );
    for l in 0..3u64 {
        put_json(&store, &format!("{l}/zarr.json"), group(json!({})));
        let n = size >> l;
        let dims: Vec<&str> = leads.iter().map(|d| d.0).chain(["lat", "lon"]).collect();
        let shape: Vec<u64> = (leads.iter().map(|d| d.1 / d.2.pow(l as u32)))
            .chain([n, n])
            .collect();
        let chunks: Vec<u64> = leads.iter().map(|_| 1).chain([n, n]).collect();
        for name in ["t2m", "sst"] {
            let path = format!("{l}/{name}");
            write_array_with(&store, &path, &dims, &shape, &chunks, values);
        }
    }
    let mut app = OctantApp::default();
    let meta = register_store(&mut app, store, TARGET);
    (app, meta)
}
