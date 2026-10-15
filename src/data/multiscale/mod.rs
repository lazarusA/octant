//! Multiscale stacks: the resolution levels of one variable, whatever format
//! described them.
//!
//! - `types.rs`: `Multiscale` (levels finest first, each with its variable,
//!   shape and per-dimension `factor`/`offset` on level 0's index grid),
//!   checked by `Multiscale::new`; `LevelRef` on `VariableInfo::multiscale`.
//! - `mapping.rs`: index ranges between level 0 and any level, the same for
//!   every dimension.
//! - Adapters, each computing only factors and offsets: `ome.rs` (OME-Zarr
//!   `multiscales`), `geozarr.rs` (zarr-conventions `multiscales` `layout`
//!   v0.1 with `spatial:transform`), `geotiff.rs` (reduced-resolution IFDs).
//!
//! Every level stays a variable of its own, with its own coordinates; a
//! stack only groups them.

mod mapping;
pub mod types;

use std::sync::Arc;

use crate::data::metadata::VariableInfo;

pub use types::{LevelRef, Multiscale, MultiscaleError, MultiscaleKind, ScaleLevel};

/// Points every variable of `set`'s levels at its level. Variables already
/// in another stack keep it.
pub fn link(set: Multiscale, variables: &mut [VariableInfo]) {
    let set = Arc::new(set);
    for var in variables.iter_mut().filter(|v| v.multiscale.is_none()) {
        if let Some(level) = set.level_of(&var.name) {
            var.multiscale = Some(LevelRef {
                set: set.clone(),
                level,
            });
        }
    }
}
