//! A worker's inspection on the wire (`ToWorker::Inspect`, `FromWorker::Inspected`): the
//! metadata as JSON, its coordinates' numbers as one `f64` part (JSON cannot hold NaN).

use serde::{Deserialize, Serialize};

use crate::data::metadata::{CoordValues, DatasetMetadata};

/// One dimension's coordinates, its numbers taken out.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum CoordHeader {
    Regular {
        start: f64,
        step: f64,
        len: usize,
    },
    /// That many numbers of the `f64` part.
    Values(usize),
    Labels(Vec<String>),
    Endpoints {
        first: f64,
        last: f64,
        len: usize,
    },
}

/// Metadata without its coordinates, which follow under their keys.
#[derive(Serialize, Deserialize, Debug)]
pub struct MetadataHeader {
    pub metadata: DatasetMetadata,
    pub coordinates: Vec<(String, CoordHeader)>,
}

/// Equal when they serialize alike (the metadata itself has no `PartialEq`).
impl PartialEq for MetadataHeader {
    fn eq(&self, other: &Self) -> bool {
        serde_json::to_value(self).ok() == serde_json::to_value(other).ok()
    }
}

impl MetadataHeader {
    /// `metadata` as its header and the numbers of its uneven coordinates.
    pub fn split(mut metadata: DatasetMetadata) -> (Self, Vec<f64>) {
        let mut numbers = Vec::new();
        let coordinates = std::mem::take(&mut metadata.dimension_coordinates)
            .into_iter()
            .map(|(key, values)| {
                let header = match values {
                    CoordValues::Regular { start, step, len } => {
                        CoordHeader::Regular { start, step, len }
                    }
                    CoordValues::Values(v) => {
                        numbers.extend_from_slice(&v);
                        CoordHeader::Values(v.len())
                    }
                    CoordValues::Labels(l) => CoordHeader::Labels(l.to_vec()),
                    CoordValues::Endpoints { first, last, len } => {
                        CoordHeader::Endpoints { first, last, len }
                    }
                };
                (key, header)
            })
            .collect();
        (
            Self {
                metadata,
                coordinates,
            },
            numbers,
        )
    }

    /// The metadata [`split`](Self::split) took apart, checked against its numbers.
    pub fn join(self, numbers: &[f64]) -> Result<DatasetMetadata, String> {
        let mut metadata = self.metadata;
        let mut rest = numbers;
        for (key, header) in self.coordinates {
            let values = match header {
                CoordHeader::Regular { start, step, len } => {
                    CoordValues::Regular { start, step, len }
                }
                CoordHeader::Values(len) => {
                    if len > rest.len() {
                        return Err(format!("Coordinate '{key}' runs past its numbers"));
                    }
                    let (values, after) = rest.split_at(len);
                    rest = after;
                    CoordValues::Values(values.into())
                }
                CoordHeader::Labels(l) => CoordValues::Labels(l.into()),
                CoordHeader::Endpoints { first, last, len } => {
                    CoordValues::Endpoints { first, last, len }
                }
            };
            metadata.dimension_coordinates.insert(key, values);
        }
        if !rest.is_empty() {
            return Err("Coordinate numbers left over".into());
        }
        Ok(metadata)
    }
}
