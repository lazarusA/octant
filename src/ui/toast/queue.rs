//! The toasts on screen: merging repeats, bounding the stack, expiring.

use std::time::Duration;

use web_time::Instant;

use super::model::{Notice, Toast};

/// Most toasts shown at once.
pub const MAX_TOASTS: usize = 4;
/// A toast fades out over its last this long.
pub const FADE: Duration = Duration::from_millis(500);

#[derive(Debug, Default)]
pub struct Toasts {
    items: Vec<Toast>,
    next_id: u64,
    last_tick: Option<Instant>,
}

impl Toasts {
    /// Oldest first.
    pub fn items(&self) -> &[Toast] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Shows `notice`. The same notice already on screen counts once more
    /// and starts its time again instead. When the stack is full, the oldest
    /// info or success toast goes first, then warnings, then errors.
    pub fn push(&mut self, notice: Notice) {
        let lifetime = notice.severity.lifetime();
        if let Some(toast) = self.items.iter_mut().find(|t| t.notice == notice) {
            toast.count = toast.count.saturating_add(1);
            toast.remaining = lifetime;
            return;
        }
        self.next_id += 1;
        self.items.push(Toast {
            id: self.next_id,
            notice,
            count: 1,
            remaining: lifetime,
        });
        while self.items.len() > MAX_TOASTS {
            let Some(drop) = self
                .items
                .iter()
                .enumerate()
                .min_by_key(|(i, t)| (t.notice.severity.keep_rank(), *i))
                .map(|(i, _)| i)
            else {
                break;
            };
            self.items.remove(drop);
        }
    }

    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|t| t.id != id);
    }

    /// Runs the clock to `now`: toasts not `hovered` (by position in
    /// `items`) lose the time since the last tick, and expired ones go.
    pub fn tick(&mut self, now: Instant, hovered: &[bool]) {
        let elapsed = self
            .last_tick
            .map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
        self.last_tick = Some(now);
        for (i, toast) in self.items.iter_mut().enumerate() {
            if !hovered.get(i).copied().unwrap_or(false) {
                toast.remaining = toast.remaining.saturating_sub(elapsed);
            }
        }
        self.items.retain(|t| !t.remaining.is_zero());
        if self.items.is_empty() {
            self.last_tick = None;
        }
    }

    /// When the next frame must be drawn: right away while one fades, else
    /// when the first one starts fading. `None` with no toasts.
    pub fn next_repaint(&self) -> Option<Duration> {
        self.items
            .iter()
            .map(|t| t.remaining.saturating_sub(FADE))
            .min()
    }
}

/// Opacity of a toast with `remaining` time: 1, falling to 0 over `FADE`.
pub fn opacity(remaining: Duration) -> f32 {
    (remaining.as_secs_f32() / FADE.as_secs_f32()).clamp(0.0, 1.0)
}
