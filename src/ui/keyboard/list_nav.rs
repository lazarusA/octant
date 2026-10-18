//! Up, Down, Home and End between the focusable items of a list drawn in
//! order (the catalog's cards), without collecting them: each item is
//! offered as it draws, and the move lands once the list has drawn.

use super::focus;
use egui::{Context, Id, Key, Response};

pub struct ListNav {
    focused: Option<Id>,
    key: Option<Key>,
    first: Option<Id>,
    previous: Option<Id>,
    take_next: bool,
    to_end: bool,
    target: Option<Id>,
}

impl ListNav {
    pub fn new(ctx: &Context) -> Self {
        Self {
            focused: ctx.memory(|m| m.focused()),
            key: focus::pressed(ctx, [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End]),
            first: None,
            previous: None,
            take_next: false,
            to_end: false,
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
        self.first = self.first.or(Some(id));
        if Some(id) == self.focused {
            focus::claim_arrows(&response.ctx, id);
            match self.key {
                Some(Key::ArrowUp) => self.target = self.previous,
                Some(Key::ArrowDown) => self.take_next = true,
                Some(Key::Home) => self.target = self.first,
                Some(Key::End) => self.to_end = true,
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
        let target = if self.to_end {
            self.previous
        } else {
            self.target
        };
        if let Some(id) = target {
            focus::request(ctx, id);
        }
    }
}
