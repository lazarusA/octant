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

pub mod geotiff;
pub mod geozarr;
pub mod geozarr_layout;
mod mapping;
pub mod ome;
pub mod types;

#[cfg(test)]
mod geozarr_tests;
#[cfg(test)]
mod tests;

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

/// Links `levels` as a stack when they form one; otherwise logs why and
/// leaves the variables as they are.
pub(crate) fn try_link(
    name: String,
    dims: Vec<String>,
    levels: Vec<ScaleLevel>,
    kind: MultiscaleKind,
    variables: &mut [VariableInfo],
) {
    let Some(first) = levels.first().map(|l| l.path.clone()) else {
        return;
    };
    match Multiscale::new(name, dims, levels, kind) {
        Ok(set) => link(set, variables),
        Err(MultiscaleError::TooFewLevels) => {}
        Err(e) => log::warn!("The levels of '{first}' are not a multiscale stack: {e}"),
    }
}
