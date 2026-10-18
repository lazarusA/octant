//! Up, Down, Home and End between the focusable items of a list drawn in
//! order (the catalog's cards), without collecting them: each item is
//! offered as it draws, and the move lands once the list has drawn. A search
//! field over a list (`Above`) leads into it: Down in the field enters the
//! list, Up on its first item returns to the field.

use super::focus;
use egui::{Context, Id, Key, Response};

/// The search field drawn over a list this frame.
#[derive(Clone, Copy, Debug)]
pub struct Above {
    pub id: Id,
    /// Down was pressed in it: focus enters the list's first item.
    pub enter: bool,
}

impl Above {
    /// Reads the search field `response`.
    pub fn of(response: &Response) -> Self {
        let down = || focus::pressed(&response.ctx, [Key::ArrowDown]).is_some();
        Self {
            id: response.id,
            enter: response.has_focus() && down(),
        }
    }
}

pub struct ListNav {
    focused: Option<Id>,
    key: Option<Key>,
    first: Option<Id>,
    previous: Option<Id>,
    above: Option<Id>,
    take_next: bool,
    to_end: bool,
    target: Option<Id>,
}

impl ListNav {
    /// A list under the search field `above`, if any.
    pub fn new(ctx: &Context, above: Option<Above>) -> Self {
        Self {
            focused: ctx.memory(|m| m.focused()),
            key: focus::pressed(ctx, [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End]),
            first: None,
            previous: None,
            above: above.map(|a| a.id),
            take_next: above.is_some_and(|a| a.enter),
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
                Some(Key::ArrowUp) => self.target = self.previous.or(self.above),
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
