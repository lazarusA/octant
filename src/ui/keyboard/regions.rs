//! The window's regions in visual order, which F6 and Shift+F6 cycle
//! through and F10 jumps into: egui's Tab follows drawing order (the bottom
//! bar is drawn before the panels above it), so regions are the way across.
//! Each region registers the widget focus enters it at while it draws; the
//! jump reads what the last frame registered.

use super::focus;
use egui::{Context, Id, LayerId, Rect, Response};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    TopBar,
    Canvas,
    BottomBar,
}

/// Regions, in the order the variants are declared: visual order.
const COUNT: usize = 3;

/// Parts one region may span (the colorbars: the base layer's and four
/// overlays').
const PARTS: usize = 5;

/// A region as drawn: where focus enters it, and the areas it covers.
#[derive(Clone, Copy, Default)]
struct Slot {
    entry: Option<Id>,
    parts: [Option<(LayerId, Rect)>; PARTS],
}

impl Slot {
    fn contains(&self, focus: focus::LastFocus) -> bool {
        self.parts
            .iter()
            .flatten()
            .any(|&(layer, rect)| layer == focus.layer && rect.contains(focus.rect.center()))
    }
}

#[derive(Clone, Copy, Default)]
struct Regions {
    drawing: [Slot; COUNT],
    drawn: [Slot; COUNT],
}

fn regions_id() -> Id {
    Id::unique("keyboard_regions")
}

/// Call once per frame before any region draws: what the last frame drew
/// becomes what jumps read.
pub fn begin_frame(ctx: &Context) {
    ctx.data_mut(|d| {
        let regions = d.get_temp_mut_or_default::<Regions>(regions_id());
        regions.drawn = std::mem::take(&mut regions.drawing);
    });
}

/// Records that `region` drew over `area` of `entry`'s layer this frame;
/// its first registered `entry` is where focus enters it.
pub fn register(region: Region, entry: &Response, area: Rect) {
    entry.ctx.data_mut(|d| {
        let slot = &mut d.get_temp_mut_or_default::<Regions>(regions_id()).drawing[region as usize];
        slot.entry.get_or_insert(entry.id);
        if let Some(free) = slot.parts.iter_mut().find(|p| p.is_none()) {
            *free = Some((entry.layer_id, area));
        }
    });
}

fn drawn(ctx: &Context) -> [Slot; COUNT] {
    ctx.data(|d| {
        d.get_temp::<Regions>(regions_id())
            .map(|r| r.drawn)
            .unwrap_or_default()
    })
}

/// Focuses the next region after (or before) the one holding focus,
/// skipping regions not drawn; from outside every region, the first (or
/// last) one.
pub fn cycle(ctx: &Context, forward: bool) {
    let slots = drawn(ctx);
    let current = focus::last(ctx).and_then(|f| slots.iter().position(|s| s.contains(f)));
    let nth = |k: usize| match (forward, current) {
        (true, Some(c)) => (c + k) % COUNT,
        (true, None) => k - 1,
        (false, Some(c)) => (c + COUNT - k % COUNT) % COUNT,
        (false, None) => COUNT - k,
    };
    let target = (1..=COUNT).map(nth).find_map(|i| slots[i].entry);
    if let Some(id) = target {
        focus::request(ctx, id);
    }
}

/// Focuses `region`'s entry, when it was drawn last frame.
pub fn enter(ctx: &Context, region: Region) {
    if let Some(id) = drawn(ctx)[region as usize].entry {
        focus::request(ctx, id);
    }
}
