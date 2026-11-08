use std::collections::HashMap;
use std::sync::Arc;

use super::wire::{BlockHeader, FromWorker, LocalRead, ToWorker};
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::data::{DataSource, DataSourceKind};
use crate::i18n::Language;

fn block() -> OctantBlock {
    let coordinates = HashMap::from([
        ("lat".to_owned(), vec![10.0, f64::NAN]),
        ("lon".to_owned(), vec![0.0, 1.5, 3.0]),
    ]);
    let attributes = HashMap::from([("units".to_owned(), "K".to_owned())]);
    let values = vec![1.0, f32::NAN, -2.0, 4.0, 5.0, 6.0];
    let mut block = OctantBlock::new(
        "t2m".into(),
        vec![2, 3],
        vec!["lat".into(), "lon".into()],
        vec![4, 7],
        values,
        coordinates,
        attributes,
    );
    block.flipped_dims = vec!["lat".into()];
    block
}

/// Coordinates compared bit for bit, so NaN equals NaN.
fn coordinate_bits(block: &OctantBlock) -> HashMap<String, Vec<u64>> {
    let bits = |v: &Vec<f64>| v.iter().map(|x| x.to_bits()).collect();
    block
        .coordinates
        .iter()
        .map(|(k, v)| (k.clone(), bits(v)))
        .collect()
}

#[test]
fn a_block_comes_back_whole() {
    let sent = block();
    let (header, values, numbers) = BlockHeader::split(sent.clone());
    let text = serde_json::to_string(&FromWorker::Block { id: 3, header }).unwrap();
    let FromWorker::Block { id, header } = serde_json::from_str(&text).unwrap() else {
        panic!("not a block");
    };
    let back = header.join(values, &numbers).unwrap();
    assert_eq!(id, 3);
    assert_eq!(back.variable_name, sent.variable_name);
    assert_eq!(back.shape, sent.shape);
    assert_eq!(back.strides, sent.strides);
    assert_eq!(back.dimension_names, sent.dimension_names);
    assert_eq!(back.origin, sent.origin);
    assert_eq!(back.flipped_dims, sent.flipped_dims);
    assert_eq!(back.attributes, sent.attributes);
    assert_eq!((back.min_value, back.max_value), (-2.0, 6.0));
    assert_eq!(coordinate_bits(&back), coordinate_bits(&sent));
    let bits = |b: &OctantBlock| b.values.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&back), bits(&sent));
}

#[test]
fn a_block_without_finite_values_keeps_its_nan_range() {
    let block = OctantBlock::new(
        "x".into(),
        vec![1],
        vec!["i".into()],
        vec![0],
        vec![f32::NAN],
        HashMap::new(),
        HashMap::new(),
    );
    let (min, max) = (block.min_value, block.max_value);
    let (header, values, numbers) = BlockHeader::split(block);
    let back = header.join(values, &numbers).unwrap();
    assert_eq!(back.min_value.to_bits(), min.to_bits());
    assert_eq!(back.max_value.to_bits(), max.to_bits());
}

#[test]
fn mismatched_parts_are_refused() {
    let (header, values, numbers) = BlockHeader::split(block());
    let text = serde_json::to_string(&header).unwrap();
    let header = || serde_json::from_str::<BlockHeader>(&text).unwrap();
    let short: Arc<[f32]> = values[..5].into();
    assert!(header().join(short, &numbers).is_err());
    assert!(header().join(values.clone(), &numbers[..1]).is_err());
    assert!(
        header()
            .join(values.clone(), &numbers[..numbers.len() - 1])
            .is_err()
    );
    let mut long = numbers.clone();
    long.push(0.0);
    assert!(header().join(values, &long).is_err());
}

#[test]
fn messages_to_the_worker_read_back() {
    let messages = [
        ToWorker::Budget { bytes: 64 << 20 },
        ToWorker::Load {
            id: 9,
            source: DataSource::new("id", DataSourceKind::RemoteZarr, "https://x/a.zarr", "a"),
            slice: SliceRequest::new(
                "t2m",
                vec![
                    DimensionSelection::Index(2),
                    DimensionSelection::range(0, 8),
                ],
            ),
            language: Language::default(),
        },
        ToWorker::Release {
            uri: "local://1/a.zarr".into(),
        },
        ToWorker::ReadDone {
            id: 1,
            error: Some("HTTP 404 reading 'local://1/a.zarr/x'".into()),
        },
    ];
    for message in messages {
        let text = serde_json::to_string(&message).unwrap();
        assert_eq!(serde_json::from_str::<ToWorker>(&text).unwrap(), message);
    }
    let read = FromWorker::Read {
        id: 2,
        url: "local://1/a.tif".into(),
        read: LocalRead::Range {
            offset: 8,
            length: 16,
        },
    };
    let text = serde_json::to_string(&read).unwrap();
    assert_eq!(serde_json::from_str::<FromWorker>(&text).unwrap(), read);
}
