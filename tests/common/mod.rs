//! Helpers shared by the integration tests.

use std::time::{Duration, Instant};

use octant::app::OctantApp;

/// Polls block results until no request is pending (at most 20 s).
pub fn drain(app: &mut OctantApp) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        app.poll_block_prefetch_results();
        let pending = app.block_prefetcher.pending_count();
        if pending == 0 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "prefetcher did not finish: {pending} pending, playing = {}",
            app.is_playing
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
