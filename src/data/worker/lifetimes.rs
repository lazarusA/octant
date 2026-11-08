//! When a data worker lets go of a store (`host.rs`). Its loads read the
//! store's registry by URL, so a store the page releases while loads run on
//! it is kept until the last of them ends; a load asked for meanwhile (its
//! dataset opened again) cancels the release.

use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Lifetimes {
    /// The loads running per store URL.
    running: HashMap<String, usize>,
    /// Store URLs released while loads ran on them.
    released: HashSet<String>,
}

impl Lifetimes {
    /// A load on `uri` arrives, cancelling a release waiting for its loads.
    pub fn begin(&mut self, uri: &str) {
        self.released.remove(uri);
        *self.running.entry(uri.to_owned()).or_default() += 1;
    }

    /// The page released `uri`: whether to let it go now, as no load runs on
    /// it (else the last one to end does).
    pub fn release(&mut self, uri: &str) -> bool {
        if self.running.contains_key(uri) {
            self.released.insert(uri.to_owned());
            return false;
        }
        true
    }

    /// A load on `uri` ended: whether to let it go now, being released and
    /// the last load.
    pub fn end(&mut self, uri: &str) -> bool {
        let Some(count) = self.running.get_mut(uri) else {
            return false;
        };
        *count = count.saturating_sub(1);
        if *count > 0 {
            return false;
        }
        self.running.remove(uri);
        self.released.remove(uri)
    }

    /// Whether `uri` was released and waits for its loads: an opening store
    /// is then not kept.
    pub fn released(&self, uri: &str) -> bool {
        self.released.contains(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::Lifetimes;

    const URI: &str = "https://x/a.zarr";

    #[test]
    fn a_store_no_load_runs_on_goes_at_once() {
        let mut lifetimes = Lifetimes::default();
        assert!(lifetimes.release(URI));
        lifetimes.begin(URI);
        assert!(!lifetimes.end(URI), "never released since");
    }

    #[test]
    fn a_release_waits_for_the_last_load() {
        let mut lifetimes = Lifetimes::default();
        lifetimes.begin(URI);
        lifetimes.begin(URI);
        assert!(!lifetimes.release(URI));
        assert!(lifetimes.released(URI));
        assert!(!lifetimes.end(URI), "one load still runs");
        assert!(lifetimes.end(URI), "the last load lets it go");
        assert!(!lifetimes.released(URI));
    }

    #[test]
    fn a_load_after_the_release_cancels_it() {
        let mut lifetimes = Lifetimes::default();
        lifetimes.begin(URI);
        assert!(!lifetimes.release(URI));
        lifetimes.begin(URI);
        assert!(!lifetimes.released(URI));
        assert!(!lifetimes.end(URI));
        assert!(!lifetimes.end(URI), "opened again: kept");
    }

    #[test]
    fn stores_are_released_apart() {
        let mut lifetimes = Lifetimes::default();
        lifetimes.begin(URI);
        lifetimes.begin("https://x/b.zarr");
        assert!(!lifetimes.release(URI));
        assert!(!lifetimes.end("https://x/b.zarr"));
        assert!(lifetimes.end(URI));
    }
}
