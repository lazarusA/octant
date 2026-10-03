//! Shared layout metrics and typography for the hero landing page.

/// Minimum empty space kept on each side of hero content.
pub const GUTTER: f32 = 16.0;
/// Widest the intake bar, banners and hints grow on large windows.
const CONTENT_MAX_W: f32 = 480.0;
/// Narrowest content width before the page simply clips.
const CONTENT_MIN_W: f32 = 160.0;

/// Font size for sample chips, banners, hints and status lines.
pub const BODY_FONT: f32 = 13.0;
/// Font size for secondary labels such as the chip `try:` prefix.
pub const SMALL_FONT: f32 = 12.0;

/// Outer width of a hero content block (intake bar, banners) for a column of
/// `avail` width, leaving at least [`GUTTER`] on each side.
pub fn content_width(avail: f32) -> f32 {
    (avail - 2.0 * GUTTER)
        .min(CONTENT_MAX_W)
        .max(CONTENT_MIN_W.min(avail))
}

/// Wordmark cell size in points, growing with the window (word is 35 cells wide).
pub fn wordmark_cell(avail: f32) -> f32 {
    if avail < 380.0 {
        5.0
    } else if avail < 520.0 {
        6.0
    } else {
        7.0
    }
}

/// First of `options` (ordered longest first) whose monospace width at
/// `size` fits in `max_w`; falls back to the last, shortest option.
///
/// Monospace glyphs share one advance, so width is measured from a single
/// glyph times the character count, with no string or galley allocation.
pub fn fit_text<'a>(ui: &egui::Ui, options: &[&'a str], size: f32, max_w: f32) -> &'a str {
    let font = egui::FontId::monospace(size);
    let advance = ui.ctx().fonts_mut(|f| f.glyph_width(&font, 'M'));
    options
        .iter()
        .copied()
        .find(|text| text.chars().count() as f32 * advance <= max_w)
        .or_else(|| options.last().copied())
        .unwrap_or_default()
}

/// Vertical gaps between hero sections, at full window height.
pub mod gap {
    /// Octant cube to wordmark.
    pub const CUBE_TITLE: f32 = 20.0;
    /// Wordmark to intake bar.
    pub const TITLE_INTAKE: f32 = 32.0;
    /// Intake bar to its helper caption / drag feedback banner.
    pub const INTAKE_HINT: f32 = 8.0;
    /// Helper caption to sample chips.
    pub const HINT_CHIPS: f32 = 28.0;
    /// Sample chips to the loading / loaded status line.
    pub const CHIPS_STATUS: f32 = 24.0;
    /// Between wrapped rows of sample chips.
    pub const CHIP_ROWS: f32 = 8.0;
}

/// Scale a [`gap`] for a hero of `avail_h` height: full size from 720 px tall,
/// shrinking to 60% on short windows so content stays on screen.
pub fn vspace(base: f32, avail_h: f32) -> f32 {
    base * (avail_h / 720.0).clamp(0.6, 1.0)
}
