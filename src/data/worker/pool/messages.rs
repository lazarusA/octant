//! The page taking what a data worker sends: hellos, blocks, inspections, failures,
//! progress, notices and the reads it asks of the page's local files.

use js_sys::Array;

use super::super::js;
use super::super::slot::answer_read;
use super::super::wire::{FromWorker, ToWorker};
use super::{Outcome, Pool};
use crate::data::blocks::memory_budget::DOWNLOAD_BUDGET;

impl Pool {
    /// Takes `message` from the worker at `index`, unless it comes from an
    /// earlier start (`serial`) of it.
    pub(in crate::data::worker) fn receive(
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
        super::super::slot::clear_timer(slot.timer);
        js::post(&slot.worker, &ToWorker::Budget { bytes: share }, &[]);
        for message in std::mem::take(&mut slot.early) {
            js::post(&slot.worker, &message, &[]);
        }
    }

    pub(super) fn finish(&mut self, index: usize, id: u64, outcome: Outcome) {
        if let Some(slot) = self.slots[index].as_mut() {
            slot.jobs.remove(&id);
        }
        if let Some(tx) = self.pending.remove(&id) {
            let _ = tx.send(outcome);
        }
    }
}
