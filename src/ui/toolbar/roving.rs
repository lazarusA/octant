//! Arrow keys within a toolbar: Left and Right move between its items in
//! visual order (the right-hand group draws right to left, so drawing order
//! is no guide), Home and End to its ends, Down opens an item's popup and
//! moves focus into it, Escape leaves the bar. Items are collected while
//! the bar draws; the keys act once it has drawn.

use crate::ui::keyboard::focus;
use crate::ui::keyboard::regions::{self, Region};
use egui::{Id, Key, Popup, Response, Ui};

/// Focusable items a bar holds at most.
const MAX: usize = 16;

/// How an item takes the arrow keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Button,
    /// A button opening the popup of this id.
    Menu(Id),
    /// A slider: reached by Left and Right, then its own arrows move it.
    Slider,
}

impl Stop {
    /// The stop of a button whose response opens an egui popup.
    pub fn menu(response: &Response) -> Self {
        Stop::Menu(Popup::default_response_id(response))
    }
}

#[derive(Clone, Copy, Debug)]
struct Item {
    id: Id,
    left: f32,
    stop: Stop,
}

/// A bar's items as drawn this frame.
pub struct BarNav {
    items: [Option<Item>; MAX],
}

impl Default for BarNav {
    fn default() -> Self {
        Self { items: [None; MAX] }
    }
}

impl BarNav {
    pub fn push(&mut self, response: &Response, stop: Stop) {
        if !response.sense.is_focusable() {
            return;
        }
        if let Some(free) = self.items.iter_mut().find(|i| i.is_none()) {
            *free = Some(Item {
                id: response.id,
                left: response.rect.left(),
                stop,
            });
        }
    }

    fn iter(&self) -> impl Iterator<Item = Item> + '_ {
        self.items.iter().flatten().copied()
    }

    /// The item furthest left (`leftmost`) or right among those `keep` passes.
    fn extreme(&self, leftmost: bool, keep: impl Fn(&Item) -> bool) -> Option<Item> {
        let items = self.iter().filter(keep);
        if leftmost {
            items.min_by(|a, b| a.left.total_cmp(&b.left))
        } else {
            items.max_by(|a, b| a.left.total_cmp(&b.left))
        }
    }

    /// Registers the bar drawn in `ui` as `region` (entered at its leftmost
    /// item) and moves focus within it on this frame's keys.
    pub fn finish(&self, ui: &Ui, region: Region) {
        let Some(first) = self.extreme(true, |_| true) else {
            return;
        };
        let ctx = ui.ctx();
        regions::register(ctx, region, first.id, ui.layer_id(), ui.max_rect());
        let focused = ctx.memory(|m| m.focused());
        let Some(at) = self.iter().find(|i| Some(i.id) == focused) else {
            return;
        };
        if at.stop == Stop::Slider {
            return;
        }
        focus::claim_arrows(ctx, at.id);
        let keys = [
            Key::ArrowLeft,
            Key::ArrowRight,
            Key::Home,
            Key::End,
            Key::ArrowDown,
            Key::Escape,
        ];
        let target = match focus::pressed(ctx, keys) {
            Some(Key::ArrowLeft) => self.extreme(false, |i| i.left < at.left),
            Some(Key::ArrowRight) => self.extreme(true, |i| i.left > at.left),
            Some(Key::Home) => Some(first),
            Some(Key::End) => self.extreme(false, |_| true),
            Some(Key::ArrowDown) => {
                if let Stop::Menu(popup) = at.stop {
                    Popup::open_id(ctx, popup);
                    focus::advance_next_frame(ctx);
                }
                None
            }
            Some(Key::Escape) => {
                ctx.memory_mut(|m| m.surrender_focus(at.id));
                None
            }
            _ => None,
        };
        if let Some(item) = target {
            focus::request(ctx, item.id);
        }
    }
}
