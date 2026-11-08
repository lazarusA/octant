//! Inspections in a worker, for stores only workers read (NetCDF, `netcdf/wasm.rs`).

use super::pool::{Outcome, run};
use super::wire::ToWorker;
use crate::data::DataSource;
use crate::data::metadata::DatasetMetadata;

/// Inspects `source` in a worker: stores only workers read (NetCDF).
pub async fn inspect(source: &DataSource) -> Result<DatasetMetadata, String> {
    let outcome = run(&source.uri, "", |id| ToWorker::Inspect {
        id,
        source: source.clone(),
        language: crate::i18n::language(),
    })
    .await;
    match outcome {
        Outcome::Inspected(metadata) => Ok(*metadata),
        Outcome::Failed(error) => Err(error),
        _ => Err(crate::i18n::tr(crate::i18n::DatasetKey::NeedsDataWorkers).to_owned()),
    }
}
