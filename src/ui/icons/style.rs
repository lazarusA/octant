//! Icon size steps, semantic tones and stroke weights shared by every icon.
//!
//! Tones use a cinematic teal & orange family: the hero's electric blue as the
//! accent, teals for info and success, warm oranges for warning and error. Each
//! tone has a dark-theme and a deeper light-theme value so it stays legible on
//! both.

use egui::{Color32, Visuals};

/// The only icon sizes used in the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSize {
    /// Bullets, chevrons and small status markers next to small text.
    Xs,
    /// Inline with body text: labels, list rows, framed buttons.
    Sm,
    /// Toolbar buttons, section headers and empty-state icons.
    Md,
    /// Brand logo.
    Lg,
}

impl IconSize {
    pub const ALL: [IconSize; 4] = [IconSize::Xs, IconSize::Sm, IconSize::Md, IconSize::Lg];

    #[inline]
    pub const fn px(self) -> f32 {
        match self {
            IconSize::Xs => 12.0,
            IconSize::Sm => 14.0,
            IconSize::Md => 18.0,
            IconSize::Lg => 24.0,
        }
    }
}

/// Gap between an icon and its label, for every icon-with-text widget.
pub const ICON_GAP: f32 = 5.0;

/// Base stroke width for an icon drawn in a box of side `dim`.
///
/// Fixed per size step so icons of different sizes share one optical weight;
/// larger decorative art scales linearly from the `Lg` weight.
pub fn stroke_width(dim: f32) -> f32 {
    if dim <= 13.0 {
        1.25
    } else if dim <= 16.0 {
        1.35
    } else if dim <= 21.0 {
        1.5
    } else if dim <= 28.0 {
        1.75
    } else {
        (dim * (1.75 / 24.0)).min(3.0)
    }
}

/// Semantic icon color that adapts to the current theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconTone {
    Default,
    Muted,
    Strong,
    Accent,
    Info,
    Success,
    Warning,
    Error,
}

impl IconTone {
    pub const ALL: [IconTone; 8] = [
        IconTone::Default,
        IconTone::Muted,
        IconTone::Strong,
        IconTone::Accent,
        IconTone::Info,
        IconTone::Success,
        IconTone::Warning,
        IconTone::Error,
    ];

    pub fn color(self, visuals: &Visuals) -> Color32 {
        let dark = visuals.dark_mode;
        let pick = |d: [u8; 3], l: [u8; 3]| {
            let [r, g, b] = if dark { d } else { l };
            Color32::from_rgb(r, g, b)
        };
        match self {
            IconTone::Default => visuals.text_color(),
            // Brighter than egui's weak text in dark mode so icons keep 3:1 contrast.
            IconTone::Muted => pick([112, 112, 112], [136, 136, 136]),
            IconTone::Strong => visuals.strong_text_color(),
            IconTone::Accent => pick([0, 190, 255], [0, 125, 220]),
            IconTone::Info => pick([64, 210, 222], [0, 126, 140]),
            IconTone::Success => pick([48, 204, 162], [0, 124, 94]),
            IconTone::Warning => pick([255, 130, 60], [196, 86, 16]),
            IconTone::Error => pick([255, 96, 72], [196, 44, 28]),
        }
    }

    /// Translucent version of the tone for backgrounds and glows.
    pub fn tint(self, visuals: &Visuals, alpha: u8) -> Color32 {
        let c = self.color(visuals);
        Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
    }

    /// [`Self::tint`] with a separate opacity per theme; light backgrounds
    /// usually need a fainter tint for the same visual weight.
    pub fn themed_tint(self, visuals: &Visuals, dark_alpha: u8, light_alpha: u8) -> Color32 {
        let alpha = if visuals.dark_mode {
            dark_alpha
        } else {
            light_alpha
        };
        self.tint(visuals, alpha)
    }

    pub const fn name(self) -> &'static str {
        match self {
            IconTone::Default => "Default",
            IconTone::Muted => "Muted",
            IconTone::Strong => "Strong",
            IconTone::Accent => "Accent",
            IconTone::Info => "Info",
            IconTone::Success => "Success",
            IconTone::Warning => "Warning",
            IconTone::Error => "Error",
        }
    }
}
