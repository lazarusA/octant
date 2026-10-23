//! The listening indicator: a disk that breathes with the microphone's
//! loudness, its rim rippled by [`LOBES`] soft lobes of the mel bands, under
//! one faint halo. Each level is measured against its own noise floor and
//! recent peak, so a quiet room rests as a plain circle and speech uses the
//! whole range. Drawn in greys between the theme's background and strong
//! text, so the disk is light on the dark theme and dark on the light one.

use crate::voice::spectrum::BANDS;
use std::f32::consts::{PI, TAU};

/// Frequency groups around the rim, lowest at the top, clockwise.
const LOBES: usize = 8;
const BANDS_PER_LOBE: usize = BANDS / LOBES;
/// Rim points drawn.
const POINTS: usize = 96;
/// Seconds to rise and to fall most of the way to a new level.
const ATTACK: f32 = 0.03;
const RELEASE: f32 = 0.2;
/// How fast a noise floor creeps up and a peak creeps down, per column
/// (10 ms): about 0.05 and 0.1 of the dB range per second.
const FLOOR_RISE: f32 = 0.0005;
const PEAK_FALL: f32 = 0.001;
/// The narrowest range levels are spread over, so noise is not amplified.
const MIN_RANGE: f32 = 0.15;
/// Turns per second of the rim's soft highlight.
const SPIN: f32 = 0.04;

/// A level's adaptive range, in the spectrum's 0..1 dB positions.
#[derive(Clone, Copy)]
struct Range {
    floor: f32,
    peak: f32,
}

impl Default for Range {
    fn default() -> Self {
        Self {
            floor: 1.0,
            peak: 0.0,
        }
    }
}

impl Range {
    /// Follows `raw` and gives its place between the floor and the peak.
    fn scale(&mut self, raw: f32) -> f32 {
        self.floor = raw.min(self.floor + FLOOR_RISE);
        self.peak = raw.max(self.peak - PEAK_FALL);
        ((raw - self.floor) / (self.peak - self.floor).max(MIN_RANGE)).clamp(0.0, 1.0)
    }
}

#[derive(Default)]
pub(super) struct Pulsar {
    /// Smoothed loudness and lobes, 0..1 of their ranges.
    loudness: f32,
    lobes: [f32; LOBES],
    /// What they move toward: the loudest column since the last frame.
    target_loudness: f32,
    target_lobes: [f32; LOBES],
    /// No column taken since the last frame.
    stale: bool,
    range: Range,
    lobe_ranges: [Range; LOBES],
    turn: f32,
}

impl Pulsar {
    /// Takes one spectrum column into the targets and the adaptive ranges.
    pub(super) fn push(&mut self, column: &[f32; BANDS]) {
        let mean = |bands: &[f32]| bands.iter().sum::<f32>() / bands.len() as f32;
        let stale = self.stale;
        let take = |target: &mut f32, v: f32| *target = if stale { v } else { target.max(v) };
        take(&mut self.target_loudness, self.range.scale(mean(column)));
        let lobes = self.target_lobes.iter_mut().zip(&mut self.lobe_ranges);
        for ((target, range), bands) in lobes.zip(column.chunks(BANDS_PER_LOBE)) {
            take(target, range.scale(mean(bands)));
        }
        self.stale = false;
    }

    /// Back to rest, keeping the room's ranges for the next listen.
    pub(super) fn reset(&mut self) {
        self.loudness = 0.0;
        self.lobes = [0.0; LOBES];
        self.target_loudness = 0.0;
        self.target_lobes = [0.0; LOBES];
    }

    /// Moves the levels toward their targets over `dt` seconds.
    fn animate(&mut self, dt: f32) {
        let ease = |v: &mut f32, target: f32| {
            let tau = if target > *v { ATTACK } else { RELEASE };
            *v += (target - *v) * (1.0 - (-dt / tau).exp());
        };
        ease(&mut self.loudness, self.target_loudness);
        for (v, &t) in self.lobes.iter_mut().zip(&self.target_lobes) {
            ease(v, t);
        }
        self.turn = (self.turn + dt * SPIN).fract();
        self.stale = true;
    }

    /// The lobes' level at `x` turns from the top, eased between lobes.
    fn ripple(&self, x: f32) -> f32 {
        let at = x.rem_euclid(1.0) * LOBES as f32;
        let i = (at as usize).min(LOBES - 1);
        let s = 0.5 - 0.5 * (PI * (at - i as f32)).cos();
        self.lobes[i] + (self.lobes[(i + 1) % LOBES] - self.lobes[i]) * s
    }

    pub(super) fn show(&mut self, ui: &mut egui::Ui, diameter: f32, active: bool) {
        if active {
            self.animate(ui.input(|i| i.stable_dt).min(0.1));
        }
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(diameter), egui::Sense::hover());
        let (center, outer) = (rect.center(), diameter * 0.5 - 1.0);
        let visuals = ui.visuals();
        let (ground, ink) = if active {
            (visuals.panel_fill, visuals.strong_text_color())
        } else {
            (visuals.panel_fill, visuals.weak_text_color())
        };
        // A soft highlight turning around the rim; louder is brighter.
        let (turn, glow) = (self.turn, 0.75 + 0.25 * self.loudness);
        let rim_color = |x: f32| {
            let shade = 0.7 + 0.3 * (TAU * (x - turn)).cos();
            ground.lerp_to_gamma(ink, shade * glow)
        };

        let base = outer * (0.42 + 0.28 * self.loudness);
        // The ripple grows with loudness: near silence the disk is round.
        let swell = outer * 0.25 * self.loudness.sqrt();
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(center, ground.lerp_to_gamma(ink, 0.35 * glow));
        for i in 0..POINTS {
            let x = i as f32 / POINTS as f32;
            let r = base + swell * self.ripple(x);
            let a = TAU * x - PI / 2.0;
            mesh.colored_vertex(center + r * egui::vec2(a.cos(), a.sin()), rim_color(x));
        }
        let rim = POINTS as u32;
        for i in 0..rim {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % rim);
        }

        let painter = ui.painter();
        if active && self.loudness > 0.01 {
            let halo = (base + outer * 0.4 * self.loudness).min(outer);
            let color = ink.gamma_multiply(0.35 * self.loudness);
            painter.circle_stroke(center, halo, egui::Stroke::new(1.0, color));
        }
        painter.add(mesh);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_the_room_and_ease() {
        let mut pulsar = Pulsar::default();
        // A steady hum becomes the floor: it rests near zero.
        for _ in 0..200 {
            pulsar.push(&[0.4; BANDS]);
            pulsar.animate(0.01);
        }
        assert!(pulsar.loudness < 0.05, "{}", pulsar.loudness);
        // Speech over it rises quickly, then fades slowly once it stops.
        for _ in 0..10 {
            pulsar.push(&[0.8; BANDS]);
            pulsar.animate(0.01);
        }
        let speaking = pulsar.loudness;
        assert!(speaking > 0.9, "{speaking}");
        pulsar.push(&[0.4; BANDS]);
        pulsar.animate(0.05);
        assert!(pulsar.loudness > 0.5 && pulsar.loudness < speaking);
    }

    #[test]
    fn a_louder_low_hum_still_rests_flat() {
        // Room noise is loudest in the lowest bands; each lobe has its own
        // floor, so none of them bulges in silence.
        let mut hum = [0.3; BANDS];
        hum[..BANDS_PER_LOBE].fill(0.6);
        let mut pulsar = Pulsar::default();
        for _ in 0..200 {
            pulsar.push(&hum);
            pulsar.animate(0.01);
        }
        assert!(pulsar.lobes.iter().all(|&l| l < 0.05), "{:?}", pulsar.lobes);
    }

    #[test]
    fn the_rim_wraps_smoothly() {
        let pulsar = Pulsar {
            lobes: [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5],
            ..Pulsar::default()
        };
        assert_eq!(pulsar.ripple(1.0 / LOBES as f32), 1.0);
        assert!((pulsar.ripple(0.0) - pulsar.ripple(1.0)).abs() < 1e-6);
        assert!((pulsar.ripple(-0.001) - pulsar.ripple(0.999)).abs() < 1e-6);
    }
}
