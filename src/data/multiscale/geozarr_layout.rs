//! The zarr-conventions `multiscales` v0.1 `layout`, with the `spatial:`
//! keys of its entries and group.

use serde_json::{Map, Value};

/// One `layout` entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub asset: String,
    pub derived_from: Option<String>,
    /// `transform.scale`, relative to `derived_from`.
    pub scale: Option<Vec<f64>>,
    /// `transform.translation`, in `derived_from`'s cells.
    pub translation: Option<Vec<f64>>,
    /// `spatial:transform` `[a, b, c, d, e, f]`.
    pub affine: Option<[f64; 6]>,
    /// `spatial:dimensions` (`[y, x]`).
    pub spatial_dims: Option<Vec<String>>,
}

/// A pyramid group's levels.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub entries: Vec<Entry>,
    /// The group's `spatial:dimensions`.
    pub spatial_dims: Option<Vec<String>>,
    /// `spatial:registration: "node"`: affines locate cell centres, not corners.
    pub node: bool,
}

/// A level's factor and offset per dimension, chained from the base.
#[derive(Debug, Clone, PartialEq)]
pub struct Chain {
    pub factor: Vec<f64>,
    pub offset: Vec<f64>,
}

impl Layout {
    /// The layout in a group's attributes; `None` when they hold no
    /// `multiscales` object with a `layout` of named assets.
    pub fn parse(attrs: &Map<String, Value>) -> Option<Self> {
        let layout = attrs
            .get("multiscales")?
            .as_object()?
            .get("layout")?
            .as_array()?;
        let mut entries: Vec<Entry> = layout.iter().filter_map(entry).collect();
        if entries.is_empty() {
            return None;
        }
        let root_affine = attrs.get("spatial:transform").and_then(affine);
        let base = entries
            .iter()
            .position(|e| e.derived_from.is_none())
            .unwrap_or(0);
        if entries[base].affine.is_none() {
            entries[base].affine = root_affine;
        }
        let node = attrs
            .get("spatial:registration")
            .and_then(Value::as_str)
            .is_some_and(|r| r.eq_ignore_ascii_case("node"));
        Some(Self {
            entries,
            spatial_dims: attrs.get("spatial:dimensions").and_then(names),
            node,
        })
    }

    fn find(&self, asset: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.asset == asset)
    }

    /// `entry`'s factor and offset for an array of rank `rank`, multiplying
    /// the `transform`s along its `derived_from` chain. `None` when a step
    /// lacks a scale of that rank or the chain loops.
    pub fn chain(&self, entry: &Entry, rank: usize) -> Option<Chain> {
        let mut steps = Vec::new();
        let mut at = entry;
        while let Some(parent) = at.derived_from.as_deref() {
            if steps.len() >= self.entries.len() {
                return None;
            }
            steps.push(at);
            at = self.find(parent)?;
        }
        let mut chain = Chain {
            factor: vec![1.0; rank],
            offset: vec![0.0; rank],
        };
        for step in steps.into_iter().rev() {
            let scale = step.scale.as_ref().filter(|s| s.len() == rank)?;
            let shift = step.translation.as_ref().filter(|t| t.len() == rank);
            for d in 0..rank {
                chain.offset[d] += shift.map_or(0.0, |t| t[d]) * chain.factor[d];
                chain.factor[d] *= scale[d];
            }
        }
        Some(chain)
    }

    /// Indices of the x and y dimensions among `dims`: the last and
    /// second-last `spatial:dimensions` (the entry's, else the group's),
    /// else the last two dimensions.
    pub fn spatial_dims(&self, entry: &Entry, dims: &[String]) -> (Option<usize>, Option<usize>) {
        let find = |name: &String| dims.iter().position(|d| d.eq_ignore_ascii_case(name));
        match entry.spatial_dims.as_ref().or(self.spatial_dims.as_ref()) {
            Some(n) if n.len() >= 2 => (find(&n[n.len() - 1]), find(&n[n.len() - 2])),
            _ if dims.len() >= 2 => (Some(dims.len() - 1), Some(dims.len() - 2)),
            _ => (None, None),
        }
    }

    /// Factor and offset along x (`axis` 0) or y (`axis` 1) of a level with
    /// affine `a` on the grid of the base's affine `a0`.
    pub fn affine_placement(&self, axis: usize, a0: &[f64; 6], a: &[f64; 6]) -> (f64, f64) {
        let (step, origin) = if axis == 0 { (0, 2) } else { (4, 5) };
        let (s0, s) = (a0[step], a[step]);
        if s0 == 0.0 || !s0.is_finite() {
            return (f64::NAN, 0.0);
        }
        let half = if self.node { 0.5 } else { 0.0 };
        let edge0 = a0[origin] - half * s0;
        let edge = a[origin] - half * s;
        ((s / s0).abs(), (edge - edge0) / s0)
    }
}

fn entry(value: &Value) -> Option<Entry> {
    let obj = value.as_object()?;
    let asset = obj.get("asset")?.as_str()?.trim_matches('/').to_string();
    let transform = obj.get("transform").and_then(Value::as_object);
    let numbers = |key: &str| transform.and_then(|t| t.get(key)).and_then(floats);
    Some(Entry {
        asset,
        derived_from: obj
            .get("derived_from")
            .and_then(Value::as_str)
            .map(|s| s.trim_matches('/').to_string()),
        scale: numbers("scale"),
        translation: numbers("translation"),
        affine: obj.get("spatial:transform").and_then(affine),
        spatial_dims: obj.get("spatial:dimensions").and_then(names),
    })
}

fn floats(value: &Value) -> Option<Vec<f64>> {
    value.as_array()?.iter().map(Value::as_f64).collect()
}

fn affine(value: &Value) -> Option<[f64; 6]> {
    floats(value)?.get(..6)?.try_into().ok()
}

fn names(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|v| v.as_str().map(str::to_string))
        .collect()
}
