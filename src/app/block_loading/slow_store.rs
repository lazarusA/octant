//! A procedural 4D store slowed down like a network, and optionally
//! failing, for headless loading tests (`buffer_tests`, `prefetch_tests`).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::app::{OctantApp, StoreKind};
use crate::data::backends::ProceduralBlockStore;
use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::octant_block::OctantBlock;
use crate::data::{DataSource, Dataset, SliceRequest, StoreHandle};
use crate::plots::PlotType;

pub(super) const TARGET: &str = "procedural://volume4d";

/// The procedural store, each fetch taking `delay` like a slow network;
/// with `failed`, fetches of steps past the first fail, their first step
/// recorded.
pub(super) struct SlowStore {
    inner: ProceduralBlockStore,
    delay: Duration,
    failed: Option<Arc<Mutex<Vec<usize>>>>,
}

impl BlockStore for SlowStore {
    fn backend_name(&self) -> &str {
        "slow"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        self.inner.variables()
    }

    fn inspect(&self) -> Result<crate::data::DatasetMetadata, BlockStoreError> {
        self.inner.inspect()
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        std::thread::sleep(self.delay);
        let start = request.selections.first().map_or(0, |s| s.bounds().0);
        if let Some(failed) = self.failed.as_ref().filter(|_| start > 0) {
            failed.lock().unwrap_or_else(|p| p.into_inner()).push(start);
            return Err("HTTP 403 Forbidden".into());
        }
        self.inner.fetch_block_with_progress(request, on_progress)
    }
}

/// A heatmap of the procedural 4D variable over a store whose fetches take
/// `delay`, its first step shown, one lookahead worker.
pub(super) fn slow_app(delay: Duration) -> OctantApp {
    store_app(delay, None)
}

/// [`slow_app`] over a store that may fail (`SlowStore::failed`).
pub(super) fn store_app(delay: Duration, failed: Option<Arc<Mutex<Vec<usize>>>>) -> OctantApp {
    let inner = ProceduralBlockStore::open(TARGET).expect("open procedural store");
    let meta = inner.inspect().expect("inspect");
    let kind = StoreKind::ProceduralVolume4D;
    let source_id = StoreKind::make_source_id(kind, TARGET);
    let source = DataSource::new(&source_id, kind.to_data_source_kind(), TARGET, "Store");
    let handle = StoreHandle::new(
        source.clone(),
        Arc::new(SlowStore {
            inner,
            delay,
            failed,
        }),
    );
    let mut app = OctantApp::default();
    app.dataset_manager
        .add(Dataset::new(&source_id, source, handle));
    app.selected.store_kind = kind;
    app.selected.store_target = TARGET.to_string();
    app.load_new_metadata(meta);
    app.layout.show_hero = false;
    app.selected.plot_type = PlotType::Heatmap;
    app.prefetch_threads = 1;
    app.plot_selection();
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.layers.base.data.matrix.is_none() {
        assert!(Instant::now() < deadline, "the first block never arrived");
        app.poll_block_prefetch_results();
        std::thread::sleep(Duration::from_millis(1));
    }
    app
}
