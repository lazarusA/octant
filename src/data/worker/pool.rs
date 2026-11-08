//! The page's side: a few data workers, started when a dataset starts
//! opening (`warm`). A load goes to the worker its store and variable hash
//! to, so one variable's chunks are downloaded and kept by one worker (a
//! band or a step whose chunk arrived already downloads nothing), while other
//! variables and overlays load in parallel. Messages wait until a worker
//! says hello. When the first workers never start (missing, blocked, too
//! slow to load), loads stay on the page for the session
//! ([`Outcome::Unavailable`]); a worker that stops later fails the loads it
//! ran and is started again on the next load, and one that does not start
//! while others did leaves its loads to the page. Messages from an earlier
//! start of a worker are ignored.

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
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;

/// How a load handed to the pool ended.
pub enum Outcome {
    Block(OctantBlock),
    Failed(String),
    /// An inspection (`inspect`).
    Inspected(Box<DatasetMetadata>),
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
    /// Some worker said hello: workers can run here, so one that does not
    /// start later never turns them all off.
    any_ready: bool,
    /// Places whose worker did not start although others did: their loads
    /// run on the page, never starting it again.
    page_only: HashSet<usize>,
}

thread_local! {
    pub(super) static POOL: RefCell<Pool> = RefCell::default();
}

/// Loads `request` in a worker.
pub async fn load(request: &BlockRequest) -> Outcome {
    let source = request.store.source();
    run(&source.uri, &request.slice.variable, |id| ToWorker::Load {
        id,
        source: source.clone(),
        slice: request.slice.clone(),
        language: crate::i18n::language(),
    })
    .await
}

/// Sends the job `make` builds from its id to the worker `uri`'s `variable` loads in, and
/// waits for its outcome.
pub(super) async fn run(uri: &str, variable: &str, make: impl FnOnce(u64) -> ToWorker) -> Outcome {
    let Some(answer) = POOL.with_borrow_mut(|pool| pool.submit(uri, variable, make)) else {
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

/// Starts the workers ahead of the first load (a dataset starts opening),
/// so their download and compile overlap its inspection instead of holding
/// its first plot.
pub fn warm() {
    POOL.with_borrow_mut(|pool| {
        for index in 0..pool.places() {
            if pool.broken {
                return;
            }
            if !pool.page_only.contains(&index) {
                pool.slot(index);
            }
        }
    });
}

/// How many workers to run: half the cores, two to four.
fn pool_size() -> usize {
    let cores = web_sys::window().map_or(4.0, |w| w.navigator().hardware_concurrency());
    ((cores / 2.0) as usize).clamp(2, 4)
}

impl Pool {
    fn submit(
        &mut self,
        uri: &str,
        variable: &str,
        make: impl FnOnce(u64) -> ToWorker,
    ) -> Option<oneshot::Receiver<Outcome>> {
        let index = self.place_of(uri, variable)?;
        let id = self.next_id;
        let slot = self.slot(index)?;
        slot.jobs.insert(id);
        slot.send(make(id));
        self.next_id += 1;
        let (tx, rx) = oneshot::channel();
        self.pending.insert(id, tx);
        Some(rx)
    }

    /// The worker a store's variable loads in, unless the page loads it.
    fn place_of(&mut self, uri: &str, variable: &str) -> Option<usize> {
        if self.broken {
            return None;
        }
        let places = self.places();
        let mut hasher = std::hash::DefaultHasher::new();
        (uri, variable).hash(&mut hasher);
        let index = (hasher.finish() % places as u64) as usize;
        (!self.page_only.contains(&index)).then_some(index)
    }

    /// How many workers run here, deciding it on first use.
    fn places(&mut self) -> usize {
        if self.slots.is_empty() {
            self.slots.resize_with(pool_size(), || None);
        }
        self.slots.len()
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
                    if self.any_ready {
                        self.page_only.insert(index);
                    } else {
                        self.broken = true;
                    }
                    return None;
                }
            }
        }
        self.slots[index].as_mut()
    }

    /// Takes `message` from the worker at `index`, unless it comes from an
    /// earlier start (`serial`) of it.
    pub(super) fn receive(
        &mut self,
        index: usize,
        serial: u64,
        message: FromWorker,
        parts: &Array,
    ) {
        let current = matches!(&self.slots[index], Some(s) if s.serial == serial && !s.dead);
        if !current {
            return;
        }
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
            FromWorker::Inspected { id, header } => {
                let numbers = js::f64s(parts, 0).unwrap_or_default();
                let outcome = match header.join(&numbers) {
                    Ok(metadata) => Outcome::Inspected(Box::new(metadata)),
                    Err(error) => Outcome::Failed(error),
                };
                self.finish(index, id, outcome);
            }
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
        self.any_ready = true;
        // Half the budget each: one variable's chunks stay on one worker, and
        // several workers fill theirs only when several variables load.
        let share = DOWNLOAD_BUDGET / 2;
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

    /// The worker at `index` (start `serial`) stopped: the loads it ran fail,
    /// those it never started run on the page, and when no worker ever
    /// started, no worker is used again.
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
        if !started && self.any_ready {
            self.page_only.insert(index);
        }
        if started || self.any_ready {
            log::error!("[data worker] Stopped: {why}");
            for id in jobs {
                let outcome = match started {
                    true => Outcome::Failed(format!("The data worker stopped: {why}")),
                    false => Outcome::Unavailable,
                };
                self.finish(index, id, outcome);
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
