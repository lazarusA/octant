//! Inspections in a worker, for stores only workers read (NetCDF, `netcdf/wasm.rs`).

use futures::channel::oneshot;

use super::pool::{Outcome, POOL};
use super::wire::ToWorker;
use crate::data::DataSource;
use crate::data::metadata::DatasetMetadata;

/// Inspects `source` in a worker: stores only workers read (NetCDF).
pub async fn inspect(source: &DataSource) -> Result<DatasetMetadata, String> {
    let submitted = POOL.with_borrow_mut(|pool| {
        let index = pool.place_of(&source.uri, "")?;
        let id = pool.next_id;
        pool.next_id += 1;
        pool.slot(index)?.jobs.insert(id);
        let language = crate::i18n::language();
        let (source, tx_rx) = (source.clone(), oneshot::channel());
        pool.slot(index)?.send(ToWorker::Inspect {
            id,
            source,
            language,
        });
        pool.pending.insert(id, tx_rx.0);
        Some(tx_rx.1)
    });
    let unavailable = || crate::i18n::tr(crate::i18n::DatasetKey::NeedsDataWorkers).to_owned();
    match submitted {
        Some(answer) => match answer.await {
            Ok(Outcome::Inspected(metadata)) => Ok(*metadata),
            Ok(Outcome::Failed(error)) => Err(error),
            _ => Err(unavailable()),
        },
        None => Err(unavailable()),
    }
}
