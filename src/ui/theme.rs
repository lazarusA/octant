//! Octant's adjustments to egui's default visuals.
//!
//! The dark theme keeps egui's brightness levels and moves them to one cool
//! hue: surfaces, borders, widget fills and text alike, so nothing reads as
//! neutral gray against the panels. Selection, hover and press outlines keep
//! egui's colors for contrast.

use egui::{Color32, Theme};

/// Cool near-black for the dark theme's canvas, bars and landing screen,
/// matching the web loading screen (`--bg-color` in `index.html`). Dark gray
/// rather than black keeps text from halating and the low ends of dark
/// colormaps visible.
pub const DARK_PANEL_FILL: Color32 = Color32::from_rgb(0x12, 0x14, 0x16);

/// Floating panels, popups and the hover card: the panel fill raised by about
/// 3% white, the same cool hue one step up, so they read as elevated without
/// standing apart from the canvas.
pub const DARK_WINDOW_FILL: Color32 = Color32::from_rgb(0x19, 0x1B, 0x1D);

/// Window borders and separators (the top and bottom bars' edges, lines within
/// panels): faint, in the fills' cool hue, enough to tell overlapping panels
/// apart.
pub const DARK_STROKE: Color32 = Color32::from_rgb(0x2A, 0x2E, 0x33);

/// Text fields and framed viewers, below the panel fill.
const DARK_EXTREME_BG: Color32 = Color32::from_rgb(0x0C, 0x0E, 0x10);

/// Body text (egui's gray 140).
const DARK_TEXT: Color32 = Color32::from_rgb(0x8D, 0x92, 0x98);
/// Button text (gray 215).
const DARK_BUTTON_TEXT: Color32 = Color32::from_rgb(0xC9, 0xCC, 0xD0);
/// Hovered text (gray 245).
const DARK_HOVERED_TEXT: Color32 = Color32::from_rgb(0xDD, 0xE0, 0xE3);
/// Strong text: titles, values, selected and pressed items (egui's white,
/// which glows on the dark panels).
const DARK_STRONG_TEXT: Color32 = Color32::from_rgb(0xE6, 0xE8, 0xEB);

/// Installs the dark theme's surfaces, borders, widget fills and text colors.
pub fn install(ctx: &egui::Context) {
    ctx.style_mut_of(Theme::Dark, |style| {
        let visuals = &mut style.visuals;
        visuals.panel_fill = DARK_PANEL_FILL;
        visuals.window_fill = DARK_WINDOW_FILL;
        visuals.window_stroke.color = DARK_STROKE;
        visuals.extreme_bg_color = DARK_EXTREME_BG;

        let w = &mut visuals.widgets;
        // egui gives non-interactive frames the window's fill.
        w.noninteractive.bg_fill = DARK_WINDOW_FILL;
        w.noninteractive.weak_bg_fill = DARK_WINDOW_FILL;
        w.noninteractive.bg_stroke.color = DARK_STROKE;
        w.noninteractive.fg_stroke.color = DARK_TEXT;

        w.inactive.weak_bg_fill = Color32::from_rgb(0x2F, 0x32, 0x36);
        w.inactive.bg_fill = Color32::from_rgb(0x39, 0x3C, 0x40);
        w.inactive.fg_stroke.color = DARK_BUTTON_TEXT;

        w.hovered.weak_bg_fill = Color32::from_rgb(0x3D, 0x40, 0x44);
        w.hovered.bg_fill = Color32::from_rgb(0x43, 0x46, 0x4B);
        w.hovered.fg_stroke.color = DARK_HOVERED_TEXT;

        w.active.weak_bg_fill = Color32::from_rgb(0x25, 0x28, 0x2C);
        w.active.bg_fill = Color32::from_rgb(0x34, 0x37, 0x3B);
        w.active.fg_stroke.color = DARK_STRONG_TEXT;

        w.open.weak_bg_fill = Color32::from_rgb(0x2A, 0x2D, 0x31);
        w.open.bg_fill = DARK_WINDOW_FILL;
        w.open.bg_stroke.color = DARK_STROKE;
        w.open.fg_stroke.color = DARK_BUTTON_TEXT;
    });
}
