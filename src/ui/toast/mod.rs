//! Temporary canvas notifications for warnings, errors and results.
//!
//! - `model.rs`: `Severity` (icon, tone, lifetime), `Notice`, `Toast`, `ToastAction`.
//! - `queue.rs`: `Toasts`: repeats merge into one toast with a count, at
//!   most `MAX_TOASTS` show (info and success drop first, errors last),
//!   hovering pauses a toast's time, the last `FADE` fades it out.
//! - `report.rs`: `report` for code without the app (renderers, backends,
//!   background threads); the app drains it every frame.
//! - `paint.rs`: the stack in the canvas's bottom-right corner.
//!
//! The app shows a notice with `OctantApp::notify`; both paths also log it.

mod model;
mod paint;
mod queue;
mod report;
#[cfg(test)]
mod sheet;
#[cfg(test)]
mod tests;

pub use model::{Notice, Severity, Toast, ToastAction};
pub use queue::Toasts;
pub(crate) use report::drain_reports;
pub use report::report;

use crate::app::OctantApp;

/// Draws the app's toasts over `canvas`, except while an export captures it.
pub fn show_toasts(app: &mut OctantApp, ctx: &egui::Context, canvas: egui::Rect) {
    if app.pending_export.is_some() {
        return;
    }
    let action = paint::show(ctx, &mut app.toasts, canvas, web_time::Instant::now());
    match action {
        Some(ToastAction::RevealFile(path)) => crate::export::reveal_in_file_manager(&path),
        None => {}
    }
}
