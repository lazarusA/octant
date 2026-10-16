//! Multiscale stacks: the resolution levels of one variable, whatever format
//! described them.
//!
//! - `types.rs`: `Multiscale` (levels finest first, each with its variable,
//!   shape and per-dimension `factor`/`offset` on level 0's index grid),
//!   checked by `Multiscale::new`; `LevelRef` on `VariableInfo::multiscale`.
//! - `mapping.rs`: index ranges between level 0 and any level, the same for
//!   every dimension.
//! - `pick.rs`: `fit_level`, the finest level meeting a plot's
//!   `ResolutionDemand` (screen pixels per dimension, dimensions kept whole,
//!   a cell budget); `pinned_level` for a level the user chose.
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
pub mod pick;
pub mod types;

#[cfg(test)]
mod geozarr_chain_tests;
#[cfg(test)]
mod geozarr_tests;
#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod pick_tests;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::data::metadata::VariableInfo;

pub use pick::{DimDemand, ResolutionDemand, fit_level, pinned_level};
pub use types::{LevelRef, Multiscale, MultiscaleError, ScaleLevel};

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
    variables: &mut [VariableInfo],
) {
    match Multiscale::new(name.clone(), dims, levels) {
        Ok(set) => link(set, variables),
        Err(MultiscaleError::TooFewLevels) => {}
        Err(e) => log::warn!("The levels of '{name}' are not a multiscale stack: {e}"),
    }
}
