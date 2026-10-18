//! The window's regions in visual order, which F6 and Shift+F6 cycle
//! through and F10 jumps into: egui's Tab follows drawing order (the bottom
//! bar is drawn before the panels above it), so regions are the way across.
//! Each region registers the widget focus enters it at while it draws; the
//! jump reads what the last frame registered, and a region about to appear
//! (a panel just opened) takes focus once it draws.

use super::focus;
use crate::ui::panel_layout::Panel;
use egui::{Context, Id, LayerId, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    TopBar,
    Dataset,
    Variables,
    Settings,
    Dimensions,
    Canvas,
    Colorbars,
    BottomBar,
}

/// Where keyboard focus enters a region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    At(Id),
    /// The widget drawn right after this one, as Tab from it would reach:
    /// a panel's first control, whatever its body shows.
    After(Id),
}

impl From<Id> for Entry {
    fn from(id: Id) -> Self {
        Entry::At(id)
    }
}

impl Entry {
    fn focus(self, ctx: &Context) {
        match self {
            Entry::At(id) => focus::request(ctx, id),
            Entry::After(id) => {
                focus::request(ctx, id);
                focus::advance_next_frame(ctx);
            }
        }
    }
}

/// Regions, in the order the variants are declared: visual order.
const COUNT: usize = Region::BottomBar as usize + 1;

impl Region {
    pub fn of(panel: Panel) -> Self {
        match panel {
            Panel::Dataset => Region::Dataset,
            Panel::Variables => Region::Variables,
            Panel::Settings => Region::Settings,
            Panel::Dimensions => Region::Dimensions,
        }
    }
}

/// Parts one region may span (the colorbars: the base layer's and four
/// overlays').
const PARTS: usize = 5;

/// A region as drawn: where focus enters it, and the areas it covers.
#[derive(Clone, Copy, Default)]
struct Slot {
    entry: Option<Entry>,
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
    /// A region to focus once it draws, and the pass that asked.
    pending: Option<(Region, u64)>,
}

fn regions_id() -> Id {
    Id::unique("keyboard_regions")
}

/// Call once per frame before any region draws: what the last frame drew
/// becomes what jumps read.
pub fn begin_frame(ctx: &Context) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        let regions = d.get_temp_mut_or_default::<Regions>(regions_id());
        regions.drawn = std::mem::take(&mut regions.drawing);
        // A region that did not draw in the pass after it was asked for
        // (a panel without content) gives up its claim.
        if regions.pending.is_some_and(|(_, asked)| asked + 1 < pass) {
            regions.pending = None;
        }
    });
}

/// Focuses `region` where it is entered once it draws (this frame or the
/// next): a panel opening by keyboard.
pub fn enter_when_drawn(ctx: &Context, region: Region) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Regions>(regions_id()).pending = Some((region, pass))
    });
}

/// Records that `region` drew over `area` of `layer` this frame; its first
/// registered `entry` is where focus enters it.
pub fn register(
    ctx: &Context,
    region: Region,
    entry: impl Into<Entry>,
    layer: LayerId,
    area: Rect,
) {
    let entry = entry.into();
    let enter = ctx.data_mut(|d| {
        let regions = d.get_temp_mut_or_default::<Regions>(regions_id());
        let slot = &mut regions.drawing[region as usize];
        slot.entry.get_or_insert(entry);
        if let Some(free) = slot.parts.iter_mut().find(|p| p.is_none()) {
            *free = Some((layer, area));
        }
        let enter = regions.pending.is_some_and(|(r, _)| r == region);
        if enter {
            regions.pending = None;
        }
        enter
    });
    if enter {
        entry.focus(ctx);
    }
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
    if let Some(entry) = target {
        entry.focus(ctx);
    }
}

/// Focuses `region`'s entry, when it was drawn last frame.
pub fn enter(ctx: &Context, region: Region) {
    if let Some(entry) = drawn(ctx)[region as usize].entry {
        entry.focus(ctx);
    }
}
