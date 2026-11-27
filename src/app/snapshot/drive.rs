//! A snapshot frame by frame: wait until the plot is ready and still, hide
//! the panels, export the canvas, close the window once it is written.

use web_time::{Duration, Instant};

use super::Snapshot;
use crate::app::OctantApp;
use crate::export::ExportTarget;
use crate::ui::panel_layout::Panel;
use crate::ui::toast::Severity;

/// How long the plot stays ready before it is captured (late tiles, fades).
const SETTLE: Duration = Duration::from_secs(4);
/// Waited for when nothing loads or waits: a plot that may never come.
const NO_PLOT: &str = "the plot";
/// How long nothing may load or wait without a plot: the link failed.
const STALLED: Duration = Duration::from_secs(10);
/// Resizes tried before the canvas's shape counts as close enough (the top
/// bar may change height as the window narrows).
const RESHAPES: u8 = 3;

impl OctantApp {
    /// Runs the snapshot, if any, for this frame.
    pub(crate) fn drive_snapshot(&mut self, ctx: &egui::Context) {
        let Some(mut snapshot) = self.snapshot.take() else {
            return;
        };
        ctx.request_repaint_after(Duration::from_millis(100));
        if snapshot.closing || self.snapshot_step(&mut snapshot, ctx) {
            snapshot.closing = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.snapshot = Some(snapshot);
    }

    /// One frame of `s`; whether the window should close.
    fn snapshot_step(&mut self, s: &mut Snapshot, ctx: &egui::Context) -> bool {
        if let Some(error) = self.read_new_toasts(s) {
            eprintln!("octant: {error}");
            return true;
        }
        if s.requested && self.pending_export.is_none() {
            return true;
        }
        let waiting = self.snapshot_waiting_for();
        if timed_out(s, waiting) {
            return true;
        }
        if s.requested {
            return false;
        }
        self.shape_window(s, ctx);
        let stalled = waiting == Some(NO_PLOT);
        let since = stalled.then(|| *s.stalled_since.get_or_insert_with(Instant::now));
        s.stalled_since = since;
        if since.is_some_and(|t| t.elapsed() > STALLED) {
            eprintln!("octant: nothing was plotted (a share link with var= plots)");
            return true;
        }
        if waiting.is_some() {
            s.ready_since = None;
            return false;
        }
        egui::Popup::close_all(ctx);
        for panel in Panel::ALL {
            self.show_panel(panel, false);
        }
        if !s.request.colorbar {
            for layer in self.layers.iter_mut() {
                layer.colorbar.shown = false;
            }
        }
        let since = *s.ready_since.get_or_insert_with(Instant::now);
        if since.elapsed() > SETTLE {
            self.export_settings.format = s.request.format;
            self.export_settings.target = ExportTarget::FullCanvas;
            self.request_canvas_export(s.request.path.clone(), false);
            if let Some(export) = self.pending_export.as_mut() {
                export.size = s.request.size;
            }
            s.requested = true;
        }
        false
    }

    /// With `--size`, narrows or lowers the window so the canvas has the
    /// image's shape: the crop then cuts nothing off a plot that fills the
    /// canvas. Only shrinks (the screen may clamp a larger window), and
    /// measures the top bar rather than assuming its height.
    fn shape_window(&self, s: &mut Snapshot, ctx: &egui::Context) {
        let Some([w, h]) = s.request.size else {
            return;
        };
        if s.reshaped >= RESHAPES || !self.has_plot() {
            return;
        }
        let [px_w, px_h] = self.layout.canvas_px.map(|p| p as f32);
        let canvas = egui::vec2(px_w, px_h) / ctx.pixels_per_point();
        let shape = w as f32 / h as f32;
        let inner = ctx.input(|i| i.viewport().inner_rect);
        let Some(inner) = inner.filter(|_| canvas.y >= 1.0) else {
            return;
        };
        // The canvas follows a resize a frame late: act on sizes that held.
        let seen = (self.layout.canvas_px, inner.size());
        if s.measured.replace(seen) != Some(seen) {
            return;
        }
        if (canvas.x / canvas.y / shape - 1.0).abs() < 0.02 {
            return;
        }
        let fit = match canvas.x / canvas.y > shape {
            true => egui::vec2(canvas.y * shape, canvas.y),
            false => egui::vec2(canvas.x, canvas.x / shape),
        };
        let chrome = inner.size() - canvas;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(fit + chrome));
        s.reshaped += 1;
        s.ready_since = None;
    }

    /// What the plot still waits for, `NO_PLOT` when nothing loads or
    /// waits; `None` once it is ready.
    fn snapshot_waiting_for(&self) -> Option<&'static str> {
        if self.layers.base.load.block_key.is_some() {
            Some("the plot's data")
        } else if self.coordinate_loader.pending_count() > 0 {
            Some("coordinates")
        } else if self.metadata_rx.is_some() {
            Some("metadata")
        } else if !self.view_settled() {
            Some("the view's finer detail")
        } else if !self.commands_idle() {
            Some("the link's commands")
        } else if !self.has_plot() || self.hero_shown() {
            Some(NO_PLOT)
        } else {
            None
        }
    }

    /// Prints the warnings shown since the last frame; the first error's
    /// text, which ends the snapshot.
    fn read_new_toasts(&self, s: &mut Snapshot) -> Option<String> {
        let seen = s.next_toast;
        for toast in self.toasts.items().iter().filter(|t| t.id >= seen) {
            s.next_toast = toast.id + 1;
            let notice = &toast.notice;
            let sep = if notice.detail.is_empty() { "" } else { ": " };
            let text = format!("{}{sep}{}", notice.title, notice.detail);
            match notice.severity {
                Severity::Error => return Some(text),
                Severity::Warning => eprintln!("octant: warning: {text}"),
                Severity::Info | Severity::Success => {}
            }
        }
        None
    }
}

/// Whether `s` ran past its timeout, said on stderr with what it waited for:
/// the whole run, the export too, so a script never hangs on one link.
fn timed_out(s: &Snapshot, waiting: Option<&str>) -> bool {
    let timeout = s.request.timeout;
    if s.start.elapsed() <= timeout {
        return false;
    }
    let what = match (s.requested, waiting) {
        (true, _) => "the export",
        (false, Some(waiting)) => waiting,
        (false, None) => "the plot to hold still",
    };
    let secs = timeout.as_secs();
    eprintln!("octant: timed out after {secs} s waiting for {what}");
    true
}
