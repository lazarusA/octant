//! Up and Down between the focusable items of a list drawn in order (the
//! catalog's cards), without collecting them: each item is offered as it
//! draws, and the move lands once the list has drawn.

use super::focus;
use egui::{Context, Id, Key, Response};

pub struct ListNav {
    focused: Option<Id>,
    key: Option<Key>,
    previous: Option<Id>,
    take_next: bool,
    target: Option<Id>,
}

impl ListNav {
    pub fn new(ctx: &Context) -> Self {
        Self {
            focused: ctx.memory(|m| m.focused()),
            key: focus::pressed(ctx, [Key::ArrowUp, Key::ArrowDown]),
            previous: None,
            take_next: false,
            target: None,
        }
    }

    /// Offers the next item of the list.
    pub fn item(&mut self, response: &Response) {
        let id = response.id;
        if self.take_next {
            self.target = Some(id);
            self.take_next = false;
        }
        if Some(id) == self.focused {
            focus::claim_arrows(&response.ctx, id);
            match self.key {
                Some(Key::ArrowUp) => self.target = self.previous,
                Some(Key::ArrowDown) => self.take_next = true,
                _ => {}
            }
            if response.gained_focus() {
                response.scroll_to_me(None);
            }
        }
        self.previous = Some(id);
    }

    /// Moves focus to the item the keys asked for.
    pub fn finish(self, ctx: &Context) {
        if let Some(id) = self.target {
            focus::request(ctx, id);
        }
    }
}
