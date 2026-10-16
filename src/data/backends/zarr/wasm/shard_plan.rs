//! What reading a window of a sharded array needs from each shard: its
//! index (where, how long), then the byte ranges of the inner chunks the
//! window touches, merged when close.

use std::ops::Range;

use serde_json::Value;

/// Gap between byte ranges below which one request reads both.
const MERGE_GAP: u64 = 64 * 1024;
/// Largest merged request.
const MERGE_MAX: u64 = 8 * 1024 * 1024;

/// A shard index the browser reads by ranges: `bytes` (little endian)
/// entries, optionally followed by a `crc32c` checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardIndexSpec {
    /// Inner chunk and shard shapes.
    pub inner: Vec<u64>,
    pub shard: Vec<u64>,
    /// The index sits at the end of the shard (else at its start).
    pub at_end: bool,
    pub checksum: bool,
}

impl ShardIndexSpec {
    /// The spec of a Zarr v3 array metadata document whose only codec is
    /// `sharding_indexed` with a plain index; `None` for anything else
    /// (read by whole shards).
    pub fn of(meta: &Value) -> Option<Self> {
        let codecs = meta.get("codecs")?.as_array()?;
        let [sharding] = codecs.as_slice() else {
            return None;
        };
        if sharding.get("name")?.as_str()? != "sharding_indexed" {
            return None;
        }
        let config = sharding.get("configuration")?;
        let shape = |v: Option<&Value>| -> Option<Vec<u64>> {
            v?.as_array()?.iter().map(Value::as_u64).collect()
        };
        let inner = shape(config.get("chunk_shape"))?;
        let shard = shape(
            meta.get("chunk_grid")?
                .get("configuration")?
                .get("chunk_shape"),
        )?;
        let index_codecs = config.get("index_codecs")?.as_array()?;
        let name = |c: &Value| c.get("name").and_then(Value::as_str).map(str::to_string);
        let little = |c: &Value| {
            let endian = c.get("configuration").and_then(|c| c.get("endian"));
            endian.is_none_or(|e| e.as_str() == Some("little"))
        };
        let checksum = match index_codecs.as_slice() {
            [b] if name(b).as_deref() == Some("bytes") && little(b) => false,
            [b, c]
                if name(b).as_deref() == Some("bytes")
                    && little(b)
                    && name(c).as_deref() == Some("crc32c") =>
            {
                true
            }
            _ => return None,
        };
        let at_end = config
            .get("index_location")
            .and_then(Value::as_str)
            .is_none_or(|l| l == "end");
        let valid = inner.len() == shard.len() && inner.iter().all(|&c| c > 0);
        valid.then_some(Self {
            inner,
            shard,
            at_end,
            checksum,
        })
    }

    /// The spec of an opened array (see [`ShardIndexSpec::of`]).
    pub fn of_array<S: ?Sized>(array: &zarrs::array::Array<S>) -> Option<Self> {
        Self::of(&serde_json::to_value(array.metadata()).ok()?)
    }

    /// Inner chunks per shard along each dimension.
    pub fn grid(&self) -> Vec<u64> {
        self.shard
            .iter()
            .zip(&self.inner)
            .map(|(&s, &c)| s.div_ceil(c))
            .collect()
    }

    /// The index's length in bytes.
    pub fn index_len(&self) -> u64 {
        let entries: u64 = self.grid().iter().product();
        entries * 16 + if self.checksum { 4 } else { 0 }
    }

    /// The index's byte range in a shard of `size` bytes.
    pub fn index_range(&self, size: u64) -> Range<u64> {
        let len = self.index_len();
        if self.at_end {
            size.saturating_sub(len)..size
        } else {
            0..len
        }
    }

    /// Flat indices (C order) of the inner chunks of shard `shard_indices`
    /// that `window` (one range per dimension, array indices) touches.
    pub fn inner_chunks(&self, shard_indices: &[u64], window: &[Range<u64>]) -> Vec<u64> {
        let grid = self.grid();
        let mut spans = Vec::with_capacity(grid.len());
        for d in 0..grid.len() {
            let origin = shard_indices[d] * self.shard[d];
            let lo = window[d].start.max(origin) - origin;
            let hi = window[d]
                .end
                .min(origin + self.shard[d])
                .saturating_sub(origin);
            if hi <= lo {
                return Vec::new();
            }
            spans.push(lo / self.inner[d]..hi.div_ceil(self.inner[d]).min(grid[d]));
        }
        let mut flat = vec![0u64];
        for (span, &n) in spans.iter().zip(&grid) {
            flat = flat
                .iter()
                .flat_map(|&f| span.clone().map(move |i| f * n + i))
                .collect();
        }
        flat
    }
}

/// The `(offset, length)` of every inner chunk in `index` (its entries
/// read little endian; empty chunks `None`).
pub fn parse_index(spec: &ShardIndexSpec, index: &[u8]) -> Option<Vec<Option<(u64, u64)>>> {
    let entries = spec.grid().iter().product::<u64>() as usize;
    let body = index.get(..entries * 16)?;
    let (entries, _) = body.as_chunks::<16>();
    Some(
        entries
            .iter()
            .map(|e| {
                let (offset, len) = e.split_at(8);
                let word = |b: &[u8]| b.iter().rev().fold(0u64, |w, &x| (w << 8) | u64::from(x));
                let (offset, len) = (word(offset), word(len));
                (offset != u64::MAX || len != u64::MAX).then_some((offset, len))
            })
            .collect(),
    )
}

/// The byte ranges (within the shard) of the inner chunks of shard
/// `shard_indices` that `window` touches, from its index `entries`; empty
/// chunks have none.
pub fn window_ranges(
    spec: &ShardIndexSpec,
    shard_indices: &[u64],
    window: &[Range<u64>],
    entries: &[Option<(u64, u64)>],
) -> Vec<Range<u64>> {
    spec.inner_chunks(shard_indices, window)
        .into_iter()
        .filter_map(|i| entries.get(i as usize).copied().flatten())
        .map(|(offset, len)| offset..offset + len)
        .collect()
}

/// `ranges` (`offset..end`) merged where under `MERGE_GAP` apart, each
/// merged request at most `MERGE_MAX` long, in offset order.
pub fn merge(mut ranges: Vec<Range<u64>>) -> Vec<Range<u64>> {
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<Range<u64>> = Vec::with_capacity(ranges.len());
    for r in ranges.into_iter().filter(|r| r.end > r.start) {
        match merged.last_mut() {
            Some(last) if r.start <= last.end + MERGE_GAP && r.end - last.start <= MERGE_MAX => {
                last.end = last.end.max(r.end);
            }
            _ => merged.push(r),
        }
    }
    merged
}
