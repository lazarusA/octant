//! Dimension coordinates of an OME-Zarr level, from its metadata alone:
//! channel labels, and physical positions (`origin + index * scale`, see
//! [`OmeLevel::origin`](super::model::OmeLevel::origin)) along space and time axes.

use super::axes::AxisKind;
use super::model::OmeRef;
use crate::data::CoordValues;

/// `(dimension name, coordinate)` for every axis of `ome` that has one, for a
/// level of shape `shape` with dimensions `dims`.
pub fn level_coordinates<'d>(
    ome: &OmeRef,
    dims: &'d [String],
    shape: &[u64],
) -> Vec<(&'d str, CoordValues)> {
    let image = &ome.image;
    let (Some(level), Some(base)) = (ome.level(), image.levels.first()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, axis) in image.axes.iter().enumerate() {
        let (Some(dim), Some(&len)) = (dims.get(i), shape.get(i)) else {
            continue;
        };
        let len = len as usize;
        match axis.kind {
            AxisKind::Channel => {
                // Labels only when every channel has one, so they stay aligned.
                let channels = &image.channels;
                if channels.len() == len
                    && channels.iter().all(|c| !c.label.is_empty())
                    && let Some(values) =
                        CoordValues::from_labels(channels.iter().map(|c| c.label.clone()).collect())
                {
                    out.push((dim.as_str(), values));
                }
            }
            AxisKind::Space | AxisKind::Time => {
                let step = level.scale.get(i).copied().unwrap_or(1.0);
                let start = level.origin(base, i);
                let physical = axis.unit.is_some() || step != 1.0 || start != 0.0;
                if physical && step.is_finite() && step != 0.0 && start.is_finite() && len > 0 {
                    out.push((dim.as_str(), CoordValues::Regular { start, step, len }));
                }
            }
            AxisKind::Other => {}
        }
    }
    out
}
