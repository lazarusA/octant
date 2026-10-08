//! Toasts from code without access to the app (renderers, backends,
//! background threads): reported here, shown from the next frame on.

use std::sync::Mutex;

use super::model::{Notice, Severity};

/// Reports kept until the app drains them; older ones beyond this are dropped.
const MAX_PENDING: usize = 32;

static PENDING: Mutex<Vec<Notice>> = Mutex::new(Vec::new());

/// Shows a toast from anywhere (and logs it). `detail` may be empty.
pub fn report(severity: Severity, title: &str, detail: &str) {
    severity.log(title, detail);
    let mut pending = PENDING.lock().unwrap_or_else(|p| p.into_inner());
    if pending.len() >= MAX_PENDING {
        pending.remove(0);
    }
    pending.push(Notice::new(severity, title, detail));
}

/// The reports since the last drain, oldest first.
pub(crate) fn drain_reports() -> Vec<Notice> {
    let mut pending = PENDING.lock().unwrap_or_else(|p| p.into_inner());
    std::mem::take(&mut *pending)
}
