//! What a [`MetaReader`](super::MetaReader) learned of one key, and the
//! keys a web walk still needs or could not fetch.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

/// The answer for one key.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Lookup {
    /// The key's bytes, shared (cache hits copy nothing).
    Found(zarrs::storage::Bytes),
    /// The store has no such key.
    Absent,
    /// The store could not be read (a timeout, a server error): answered as
    /// missing, but never remembered beyond this reader, so the next one
    /// asks again.
    Failed,
    /// Not fetched yet (web only).
    Pending,
}

/// Keys already fetched without success, and keys still needed (web only).
#[derive(Debug, Default)]
pub struct Prefetch {
    absent: RefCell<HashSet<String>>,
    failed: RefCell<HashSet<String>>,
    pending: RefCell<BTreeSet<String>>,
}

impl Prefetch {
    /// The keys needed since the last call, emptying the list.
    pub fn take_pending(&self) -> Vec<String> {
        std::mem::take(&mut *self.pending.borrow_mut())
            .into_iter()
            .collect()
    }

    /// Records that the store has no `key`.
    pub fn mark_absent(&self, key: String) {
        self.absent.borrow_mut().insert(key);
    }

    /// Records that `key` could not be fetched.
    pub fn mark_failed(&self, key: String) {
        self.failed.borrow_mut().insert(key);
    }

    /// The answer for `key`, not in the memory store: missing, failed, or
    /// pending (and then asked for).
    pub(super) fn answer(&self, key: &str) -> Lookup {
        if self.absent.borrow().contains(key) {
            return Lookup::Absent;
        }
        if self.failed.borrow().contains(key) {
            return Lookup::Failed;
        }
        self.pending.borrow_mut().insert(key.to_string());
        Lookup::Pending
    }
}
