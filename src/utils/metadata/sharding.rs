//! Sharded Zarr v3 arrays: the inner chunks a reader can fetch on their own,
//! and the shards (stored objects) holding them.

/// The `(inner chunk, shard)` shapes of a Zarr v3 array metadata document
/// whose first codec is `sharding_indexed` (no array-to-array codec before
/// it reshapes the inner chunks); `None` for any other array.
pub fn sharding_shapes(meta: &serde_json::Value) -> Option<(Vec<u64>, Vec<u64>)> {
    let shape_of = |v: Option<&serde_json::Value>| -> Option<Vec<u64>> {
        v?.as_array()?
            .iter()
            .map(serde_json::Value::as_u64)
            .collect()
    };
    let first = meta.get("codecs")?.as_array()?.first()?;
    if first.get("name")?.as_str()? != "sharding_indexed" {
        return None;
    }
    let inner = shape_of(first.get("configuration")?.get("chunk_shape"))?;
    let shard = shape_of(
        meta.get("chunk_grid")?
            .get("configuration")?
            .get("chunk_shape"),
    )?;
    (inner.len() == shard.len() && inner.iter().all(|&c| c > 0)).then_some((inner, shard))
}
