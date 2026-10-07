//! Choosing the manifests that hold an array's chunks: each manifest reference lists the
//! chunk index ranges (`extents`) its manifest covers, so only those are fetched.

use icechunk_format::ChunkIndices;
use icechunk_format::manifest::ManifestRef;

/// The manifest references whose extents hold at least one of `chunks`, newest (last
/// appended) first, the order chunks are looked up in.
pub fn manifests_for<'a>(refs: &'a [ManifestRef], chunks: &[ChunkIndices]) -> Vec<&'a ManifestRef> {
    refs.iter()
        .rev()
        .filter(|r| chunks.iter().any(|c| r.extents.contains(&c.0)))
        .collect()
}

#[cfg(test)]
mod tests {
    use icechunk_format::manifest::ManifestExtents;

    use super::*;

    fn manifest(from: u32, to: u32) -> ManifestRef {
        ManifestRef {
            object_id: icechunk_format::ManifestId::random(),
            extents: ManifestExtents::new(&[from], &[to]),
        }
    }

    #[test]
    fn only_manifests_holding_the_chunks_are_chosen_newest_first() {
        let refs = [
            manifest(0, 10),
            manifest(10, 20),
            manifest(20, 30),
            manifest(5, 15),
        ];
        let picked = manifests_for(&refs, &[ChunkIndices(vec![12])]);
        let ids: Vec<_> = picked.iter().map(|r| &r.object_id).collect();
        assert_eq!(ids, [&refs[3].object_id, &refs[1].object_id]);
        assert!(manifests_for(&refs, &[ChunkIndices(vec![40])]).is_empty());
    }
}
