//! The page's side: a few data workers, started on the first load. A load
//! goes to the worker its block key hashes to, so retries find that worker's
//! store and chunks, and the blocks of one dataset decode in parallel.
//! Messages wait until a worker says hello. A worker that never starts
//! (missing, blocked, too slow to load) leaves loads to the page for the
//! session ([`Outcome::Unavailable`]); one that stops later fails the loads
//! it ran and is started again on the next load.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use futures::channel::oneshot;
use js_sys::Array;

use super::js;
use super::slot::{Slot, answer_read};
use super::wire::{FromWorker, ToWorker};
use crate::data::blocks::BlockRequest;
use crate::data::blocks::memory_budget::DOWNLOAD_BUDGET;
use crate::data::octant_block::OctantBlock;

/// How a load handed to the pool ended.
pub enum Outcome {
    Block(OctantBlock),
    Failed(String),
    /// No worker runs it: the page loads it itself.
    Unavailable,
}

#[derive(Default)]
pub(super) struct Pool {
    pub(super) slots: Vec<Option<Slot>>,
    next_id: u64,
    next_serial: u64,
    pending: HashMap<u64, oneshot::Sender<Outcome>>,
    /// Notices already shown, once per session.
    shown: HashSet<(String, String)>,
    /// A worker failed to start: the page loads alone from then on.
    broken: bool,
}

thread_local! {
    pub(super) static POOL: RefCell<Pool> = RefCell::default();
}

/// Loads `request` in a worker.
pub async fn load(request: &BlockRequest) -> Outcome {
    let Some(answer) = POOL.with_borrow_mut(|pool| pool.submit(request)) else {
        return Outcome::Unavailable;
    };
    answer.await.unwrap_or(Outcome::Unavailable)
}

/// Has every worker drop the stores of `uri`.
pub fn release(uri: &str) {
    POOL.with_borrow_mut(|pool| {
        for slot in pool.slots.iter_mut().flatten() {
            slot.send(ToWorker::Release {
                uri: uri.to_owned(),
            });
        }
    });
}

/// How many workers to run: half the cores, two to four.
fn pool_size() -> usize {
    let cores = web_sys::window().map_or(4.0, |w| w.navigator().hardware_concurrency());
    ((cores / 2.0) as usize).clamp(2, 4)
}

impl Pool {
    fn submit(&mut self, request: &BlockRequest) -> Option<oneshot::Receiver<Outcome>> {
        if self.broken {
            return None;
        }
        if self.slots.is_empty() {
            self.slots.resize_with(pool_size(), || None);
        }
        let mut hasher = std::hash::DefaultHasher::new();
        request.cache_key().hash(&mut hasher);
        let index = (hasher.finish() % self.slots.len() as u64) as usize;
        let id = self.next_id;
        let slot = self.slot(index)?;
        slot.jobs.insert(id);
        slot.send(ToWorker::Load {
            id,
            source: request.store.source().clone(),
            slice: request.slice.clone(),
            language: crate::i18n::language(),
        });
        self.next_id += 1;
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);
        Some(rx)
    }

    /// The live worker at `index`, started when there is none.
    fn slot(&mut self, index: usize) -> Option<&mut Slot> {
        let live = matches!(&self.slots[index], Some(slot) if !slot.dead);
        if !live {
            self.next_serial += 1;
            match Slot::start(index, self.next_serial) {
                Ok(slot) => self.slots[index] = Some(slot),
                Err(e) => {
                    log::warn!("[data worker] Not started, loading on the page: {e}");
                    self.broken = true;
                    return None;
                }
            }
        }
        self.slots[index].as_mut()
    }

    pub(super) fn receive(&mut self, index: usize, message: FromWorker, parts: &Array) {
        match message {
            FromWorker::Hello => self.hello(index),
            FromWorker::Block { id, header } => {
                let block = match (js::f32s(parts, 0), js::f64s(parts, 1)) {
                    (Some(values), Some(numbers)) => header.join(values, &numbers),
                    _ => Err("A block arrived without its numbers".to_owned()),
                };
                let outcome = block.map_or_else(Outcome::Failed, Outcome::Block);
                self.finish(index, id, outcome);
            }
            FromWorker::Failed { id, error } => self.finish(index, id, Outcome::Failed(error)),
            FromWorker::Progress { counts } => crate::data::blocks::progress::add(counts),
            FromWorker::Notice {
                severity,
                title,
                detail,
            } => {
                if self.shown.insert((title.clone(), detail.clone())) {
                    crate::ui::toast::report(severity, &title, detail);
                }
            }
            FromWorker::Read { id, url, read } => {
                if let Some(slot) = self.slots[index].as_ref() {
                    let worker = slot.worker.clone();
                    wasm_bindgen_futures::spawn_local(answer_read(worker, id, url, read));
                }
            }
        }
    }

    fn hello(&mut self, index: usize) {
        let share = DOWNLOAD_BUDGET / self.slots.len() as u64;
        let Some(slot) = self.slots[index].as_mut() else {
            return;
        };
        slot.ready = true;
        super::slot::clear_timer(slot.timer);
        js::post(&slot.worker, &ToWorker::Budget { bytes: share }, &[]);
        for message in std::mem::take(&mut slot.early) {
            js::post(&slot.worker, &message, &[]);
        }
    }

    fn finish(&mut self, index: usize, id: u64, outcome: Outcome) {
        if let Some(slot) = self.slots[index].as_mut() {
            slot.jobs.remove(&id);
        }
        if let Some(tx) = self.pending.remove(&id) {
            let _ = tx.send(outcome);
        }
    }

    /// The worker at `index` (start `serial`) stopped: its loads fail, or,
    /// when it never started, no worker is used again and they run on the page.
    pub(super) fn stopped(&mut self, index: usize, serial: u64, why: &str) {
        let Some(slot) = self.slots[index].as_mut().filter(|s| s.serial == serial) else {
            return;
        };
        if slot.dead {
            return;
        }
        let started = slot.ready;
        slot.dead = true;
        slot.worker.terminate();
        let jobs: Vec<u64> = slot.jobs.drain().collect();
        if started {
            log::error!("[data worker] Stopped: {why}");
            for id in jobs {
                let failure = format!("The data worker stopped: {why}");
                self.finish(index, id, Outcome::Failed(failure));
            }
            return;
        }
        log::warn!("[data worker] Did not start ({why}), loading on the page");
        self.broken = true;
        for slot in self.slots.iter_mut().flatten() {
            slot.dead = true;
            slot.worker.terminate();
        }
        // Dropped answers read as unavailable: the page loads them itself.
        self.pending.clear();
    }
}
