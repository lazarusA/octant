//! `LayerStack` ids and lookups by id and pending block, and per-layer volume upload marks.

use super::{Layer, LayerId, LayerRenderers, LayerStack, Source};
use crate::data::{BlockCacheKey, SliceRequest};

#[test]
fn the_base_layer_is_found_by_its_id_and_pending_block() {
    let mut stack = LayerStack::default();
    assert_eq!(stack.get(LayerId::BASE).map(Layer::id), Some(LayerId::BASE));
    assert!(stack.get_mut(LayerId::BASE).is_some());

    let key = BlockCacheKey::new("store", &SliceRequest::full_range("t2m", &[4, 2]));
    assert_eq!(stack.find_by_key(&key), None);
    stack.base.load.block_key = Some(key.clone());
    assert_eq!(stack.find_by_key(&key), Some(LayerId::BASE));
}

#[test]
fn pushed_layers_get_new_ids_and_draw_over_the_base() {
    let mut stack = LayerStack::default();
    let first = stack.push(Source::default());
    let second = stack.push(Source::default());
    assert_ne!(first, LayerId::BASE);
    assert_ne!(first, second);
    let order: Vec<LayerId> = stack.iter().map(Layer::id).collect();
    assert_eq!(order, [LayerId::BASE, first, second]);
    assert_eq!(stack.get_mut(second).map(|l| l.id()), Some(second));
}

#[test]
fn dirty_volume_planes_merge_for_both_renderers() {
    let mut renderers = LayerRenderers::default();
    renderers.mark_volume_dirty(3..3);
    assert_eq!(renderers.volume_dirty, None, "empty ranges mark nothing");
    renderers.mark_volume_dirty(4..6);
    renderers.mark_volume_dirty(1..2);
    assert_eq!(renderers.volume_dirty, Some(1..6));
    assert_eq!(renderers.point_cloud_dirty, Some(1..6));
}
