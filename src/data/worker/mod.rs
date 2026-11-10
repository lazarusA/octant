//! Data workers: the browser loads blocks in a few Web Workers (`data_worker`,
//! `src/bin/data_worker/`) instead of on the page, so fetching, decoding and
//! slicing never hold a frame, as the desktop's background threads do. The
//! page keeps deciding what to load (`blocks::prefetch`) and hands each load
//! to the pool (`pool.rs`); a worker (`host.rs`) runs the page's own
//! `BlockLoader::load_one_wasm` on stores it opens itself and sends the
//! block back (`wire.rs`, `js.rs`). Without workers the page loads alone.
//! The desktop has none.

pub mod wire;
mod wire_meta;

#[cfg(target_arch = "wasm32")]
pub mod host;
#[cfg(target_arch = "wasm32")]
mod inspect;
#[cfg(target_arch = "wasm32")]
mod js;
#[cfg(any(target_arch = "wasm32", test))]
mod lifetimes;
#[cfg(target_arch = "wasm32")]
mod pool;
#[cfg(target_arch = "wasm32")]
mod slot;

#[cfg(test)]
mod wire_tests;

pub use wire::LocalRead;

#[cfg(target_arch = "wasm32")]
pub use host::{file_from_page, read_from_page};
#[cfg(target_arch = "wasm32")]
pub use inspect::inspect;
#[cfg(target_arch = "wasm32")]
pub use pool::{release, warm};

/// Loads `request` in a worker, or on the page when it has none (stores the
/// worker cannot open, or workers that did not start).
#[cfg(target_arch = "wasm32")]
pub async fn load(
    request: &crate::data::blocks::BlockRequest,
    on_progress: crate::data::blocks::ProgressCallback<'_>,
) -> Result<crate::data::octant_block::OctantBlock, crate::data::blocks::BlockStoreError> {
    if crate::data::backends::web::WebFormat::of(request.store.source()).in_worker() {
        match pool::load(request).await {
            pool::Outcome::Block(block) => return Ok(block),
            pool::Outcome::Failed(error) => return Err(error.into()),
            pool::Outcome::Inspected(_) => return Err("An inspection answered a load".into()),
            pool::Outcome::Unavailable => {}
        }
    }
    crate::data::blocks::BlockLoader::load_one_wasm(request, on_progress).await
}

/// Whether this code runs in a data worker, not on the page.
#[cfg(target_arch = "wasm32")]
pub fn in_worker() -> bool {
    web_sys::window().is_none()
}

/// Whether this code runs in a data worker; the desktop has none.
#[cfg(not(target_arch = "wasm32"))]
pub fn in_worker() -> bool {
    false
}
