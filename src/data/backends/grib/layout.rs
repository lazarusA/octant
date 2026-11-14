//! A GRIB file's messages laid out as variables: one per parameter, level type, kind and
//! grid, its leading dimensions those its messages vary along (`axes.rs`), then the grid's
//! `y`, `x` (`grid.rs`).

use std::collections::HashMap;
use std::sync::Arc;

use gribberish::message_metadata::MessageMetadata;

use super::axes::{Axes, Axis, candidates, scalar};
use super::entry::{Entry, Item, attrs};
use super::grid::Grids;
use super::kind::kind;
use super::scan::Field;
use crate::data::backends::netcdf::reader::NcVar;
use crate::data::blocks::BlockStoreError;

/// The slots (one per combination of a variable's leading dimensions, 24 bytes) a whole
/// file may take, about 100 MB: a dense file holds about as many messages as slots, and no
/// real file comes near 4 million messages. Only messages sharing no times or levels need
/// more, since every combination of theirs gets a slot.
const MAX_SLOTS: usize = 1 << 22;

/// The messages of one variable.
struct Group {
    var: String,
    surface: String,
    kind: String,
    grid: usize,
    fields: Vec<usize>,
}

/// The variables of a file made of `fields` (data, then the coordinates and grid mappings
/// they use), and the offsets of messages left out: those at an index another message of
/// their variable already holds.
pub fn layout(fields: &[Field]) -> Result<(Vec<Entry>, Vec<u64>), BlockStoreError> {
    let mut grids = Grids::default();
    let mut groups: Vec<Group> = Vec::new();
    let mut index: HashMap<(String, String, String, usize), usize> = HashMap::new();
    for (i, field) in fields.iter().enumerate() {
        let meta = &field.meta;
        let surface = meta.first_fixed_surface_type.coordinate_name().to_owned();
        let key = (meta.var.clone(), surface, kind(meta), grids.index_of(meta));
        let at = *index.entry(key.clone()).or_insert_with(|| {
            groups.push(Group {
                var: key.0,
                surface: key.1,
                kind: key.2,
                grid: key.3,
                fields: Vec::new(),
            });
            groups.len() - 1
        });
        groups[at].fields.push(i);
    }

    let names = variable_names(&groups);
    let mut axes = Axes::default();
    let mut entries = Vec::with_capacity(groups.len());
    let mut left_out = Vec::new();
    let mut slots_left = MAX_SLOTS;
    for (group, name) in groups.iter().zip(names) {
        entries.push(variable(
            group,
            name,
            fields,
            &grids,
            &mut axes,
            (&mut left_out, &mut slots_left),
        )?);
    }
    entries.extend(axes.entries());
    for (i, grid) in grids.0.iter().enumerate() {
        entries.extend(grid.entries(&Grids::suffix(i)));
    }
    Ok((entries, left_out))
}

/// One variable: its leading dimensions, its messages by index along them, its attributes.
fn variable(
    group: &Group,
    name: String,
    fields: &[Field],
    grids: &Grids,
    axes: &mut Axes,
    (left_out, slots_left): (&mut Vec<u64>, &mut usize),
) -> Result<Entry, BlockStoreError> {
    let metas: Vec<&MessageMetadata> = group.fields.iter().map(|&i| &fields[i].meta).collect();
    let first = metas[0];
    let mut attributes = attributes(group, first);
    let mut dims = Vec::new();
    let mut leading = Vec::new();
    for axis in candidates(&metas) {
        if axis.values.len() == 1 {
            attributes.extend(scalar(&axis, first));
            continue;
        }
        dims.push((String::new(), axis.values.len()));
        leading.push(axis);
    }
    let messages = place(group, fields, &leading, left_out, slots_left)
        .ok_or_else(|| format!("'{name}': the file has too many messages to lay out"))?;
    for (dim, axis) in dims.iter_mut().zip(leading) {
        dim.0 = axes.name(axis);
    }
    let grid = &grids.0[group.grid];
    let suffix = Grids::suffix(group.grid);
    let (ydim, xdim) = grid.dims(&suffix);
    if let Some(crs) = grid.crs_name(&suffix) {
        attributes.insert("grid_mapping".into(), crs);
    }
    dims.extend([(ydim, grid.ny), (xdim, grid.nx)]);
    let var = NcVar {
        path: name,
        data_type: "float32",
        dims,
        attributes: Arc::new(attributes),
        calibration: Default::default(),
    };
    let (ny, nx) = (grid.ny, grid.nx);
    Ok(Entry {
        var,
        item: Item::Data { messages, ny, nx },
    })
}

/// The messages of `group` by their index along `leading` (row-major), the first of any
/// two at one index kept and the other's offset added to `left_out`; `None` when its
/// indices are more than the `slots_left` of the file's `MAX_SLOTS`, which they are taken from.
fn place(
    group: &Group,
    fields: &[Field],
    leading: &[Axis],
    left_out: &mut Vec<u64>,
    slots_left: &mut usize,
) -> Option<Vec<Option<(u64, u64)>>> {
    let planes = (leading.iter()).try_fold(1usize, |n, axis| n.checked_mul(axis.values.len()))?;
    *slots_left = slots_left.checked_sub(planes)?;
    let mut messages = vec![None; planes];
    for &i in &group.fields {
        let at = (leading.iter()).try_fold(0usize, |at, axis| {
            Some(at * axis.values.len() + axis.index(&fields[i].meta)?)
        });
        match at.and_then(|at| messages.get_mut(at)) {
            Some(slot @ None) => *slot = Some((fields[i].offset, fields[i].len)),
            _ => left_out.push(fields[i].offset),
        }
    }
    Some(messages)
}

/// What every message of `group` shares, as attributes (cfgrib's `GRIB_*` names).
fn attributes(group: &Group, first: &MessageMetadata) -> HashMap<String, String> {
    let surface = first.first_fixed_surface_type.to_string();
    let mut attributes = attrs(&[
        ("units", &first.units),
        ("long_name", &first.name),
        ("GRIB_shortName", &group.var),
        ("GRIB_typeOfLevel", &surface),
    ]);
    if group.kind != "instant" {
        attributes.insert("GRIB_kind".into(), group.kind.clone());
    }
    attributes
}

/// Each group's variable name: its parameter (`TMP`) when no other group shares it, else
/// with its level type, kind and grid (`TMP_hag_max`), numbered if still taken.
fn variable_names(groups: &[Group]) -> Vec<String> {
    let mut count: HashMap<&str, usize> = HashMap::new();
    for g in groups {
        *count.entry(&g.var).or_insert(0) += 1;
    }
    let mut taken: HashMap<String, usize> = HashMap::new();
    (groups.iter())
        .map(|g| {
            let mut name = g.var.clone();
            if count.get(g.var.as_str()).is_some_and(|&n| n > 1) {
                let kind = (g.kind != "instant").then_some(g.kind.as_str());
                let grid = (g.grid > 0).then(|| format!("g{}", g.grid));
                let parts = [Some(g.surface.as_str()), kind, grid.as_deref()];
                for part in parts.into_iter().flatten().filter(|p| !p.is_empty()) {
                    name = format!("{name}_{part}");
                }
            }
            let seen = taken.entry(name.clone()).or_insert(0);
            *seen += 1;
            match *seen {
                1 => name,
                n => format!("{name}_{}", n - 1),
            }
        })
        .collect()
}
