//! Array discovery heuristics across group hierarchies and consolidated metadata.

use zarrs::array::Array;
use zarrs::metadata_ext::group::consolidated_metadata::ConsolidatedMetadataMetadata;
use zarrs::storage::{ReadableStorage, ReadableWritableListableStorage};

type CoordArray = Array<dyn zarrs::storage::ReadableStorageTraits>;

/// Finds and opens an array matching any candidate path across root, group
/// children, and consolidated metadata. With the store's `consolidated`
/// metadata only candidates it lists are opened: it lists every node, so
/// the store is asked nothing (each probe a request on a remote store), and
/// a sibling group's coordinate is never taken for the variable's.
pub fn discover_coord_array(
    store: ReadableWritableListableStorage,
    consolidated: Option<&ConsolidatedMetadataMetadata>,
    candidates: &[String],
    aliases: &[&str],
) -> Option<CoordArray> {
    let readable_store: ReadableStorage = store.clone();
    if let Some(metadata) = consolidated {
        return listed_array(&readable_store, metadata, candidates);
    }
    for clean_key in candidates {
        if let Ok(arr) = crate::utils::metadata::open_or_instantiate_array_normalized(
            readable_store.clone(),
            &format!("/{clean_key}"),
        ) {
            return Some(arr);
        }
    }

    // Discover child group nodes from root if still not found
    if let Ok(root_path) = zarrs::node::NodePath::new("/")
        && let Ok(children) = zarrs::node::get_child_nodes(&store, &root_path, false)
    {
        for child in children {
            let child_group = child.path().as_str().trim_start_matches('/');
            if child_group.is_empty() {
                continue;
            }
            let child_target = format!("/{}", child_group);
            for clean_key in candidates {
                if let Ok(arr) = Array::open(
                    readable_store.clone(),
                    &format!("{}/{}", child_target, clean_key),
                ) {
                    return Some(arr);
                }
            }
            for alias in aliases {
                if let Ok(arr) = Array::open(
                    readable_store.clone(),
                    &format!("{}/{}", child_target, alias),
                ) {
                    return Some(arr);
                }
            }
        }
    }

    let group = zarrs::group::Group::open(readable_store.clone(), "/").ok()?;
    let metadata = group.consolidated_metadata()?.metadata;
    listed_array(&readable_store, &metadata, candidates)
}

/// The first of `candidates` that `metadata` lists, opened from it.
fn listed_array(
    store: &ReadableStorage,
    metadata: &ConsolidatedMetadataMetadata,
    candidates: &[String],
) -> Option<CoordArray> {
    candidates.iter().find_map(|path| {
        let key = path.trim_start_matches('/');
        let node_meta = metadata.get(key).or_else(|| metadata.get(path))?;
        crate::utils::metadata::instantiate_array_from_node_metadata(store.clone(), path, node_meta)
    })
}
