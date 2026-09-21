//! Hero animation state and octant traversal sequence.

use std::time::Duration;
use web_time::Instant;

use crate::utils::{ease_in_out_cubic, lerp3, xorshift64_f32};

// ---------------------------------------------------------------------
// The 8 octants, visited in Gray-code order so every hop moves to a
// face-adjacent neighbor (exactly one axis flips per step).
// ---------------------------------------------------------------------

pub const SEQUENCE: [[f32; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [0.0, -1.0, -1.0],
    [0.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
];

// ---------------------------------------------------------------------
// Hero State
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HeroState {
    pub input: String,
    pub current: usize,
    pub anim_from: [f32; 3],
    pub anim_to: [f32; 3],
    pub anim_start: Option<Instant>,
    pub anim_duration: Duration,
    pub next_hop_at: Instant,
    pub loading: bool,
    pub hops_left: u32,
    pub loaded: bool,
    pub source_label: String,
    pub rng_seed: u64,
}

impl Default for HeroState {
    fn default() -> Self {
        Self {
            input: String::new(),
            current: 0,
            anim_from: SEQUENCE[0],
            anim_to: SEQUENCE[0],
            anim_start: None,
            anim_duration: Duration::from_millis(1700),
            next_hop_at: Instant::now() + Duration::from_secs(9),
            loading: false,
            hops_left: 0,
            loaded: false,
            source_label: String::new(),
            rng_seed: 0x123456789abcdef0,
        }
    }
}

impl HeroState {
    pub fn begin_submit(&mut self, source_name: &str) {
        if self.loading {
            return;
        }

        self.source_label = source_name.trim().to_owned();
        self.loading = true;
        self.loaded = false;
        self.hops_left = 5;
        self.start_hop(Duration::from_millis(240));
    }

    pub fn start_hop(&mut self, duration: Duration) {
        let next = (self.current + 1) % SEQUENCE.len();
        self.anim_from = SEQUENCE[self.current];
        self.anim_to = SEQUENCE[next];
        self.anim_duration = duration;
        self.anim_start = Some(Instant::now());
    }

    pub fn schedule_next_wander(&mut self) {
        let r = xorshift64_f32(&mut self.rng_seed);
        let delay = 8.0 + r * 7.0; // 8.0 to 15.0 seconds
        self.next_hop_at = Instant::now() + Duration::from_secs_f32(delay);
    }

    /// Advance animation physics and return (filled_corner, extra_rot, extra_scale).
    pub fn update_animation(&mut self, now: Instant) -> ([f32; 3], f32, f32) {
        if let Some(start) = self.anim_start {
            let dur_secs = self.anim_duration.as_secs_f32().max(0.001);
            let t = now.duration_since(start).as_secs_f32() / dur_secs;

            if t >= 1.0 {
                self.current = (self.current + 1) % SEQUENCE.len();
                self.anim_start = None;

                if self.loading {
                    if self.hops_left > 1 {
                        self.hops_left -= 1;
                        self.start_hop(Duration::from_millis(240));
                    } else {
                        self.loading = false;
                        self.loaded = true;
                        self.schedule_next_wander();
                    }
                } else {
                    self.schedule_next_wander();
                }

                (SEQUENCE[self.current], 0.0, 1.0)
            } else {
                let eased = ease_in_out_cubic(t.clamp(0.0, 1.0));
                let f = lerp3(self.anim_from, self.anim_to, eased);
                // gentle scale-down/up + slight rotation while it's in transit
                let wobble = (std::f32::consts::PI * t).sin();
                (f, wobble * 0.16, 1.0 - wobble * 0.09)
            }
        } else {
            if !self.loading && now >= self.next_hop_at {
                self.start_hop(Duration::from_millis(1700));
            }
            (SEQUENCE[self.current], 0.0, 1.0)
        }
    }
}
