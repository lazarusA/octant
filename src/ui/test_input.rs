//! Headless egui frames for UI tests: a context with its own clock (so
//! double-clicks and fades follow the frames), pointer moves, clicks,
//! drags and touch taps, and the areas' rects. Each call takes the frame's
//! UI as a closure; build it per call (`&mut run(&mut app)`) so the app
//! stays free between frames.

use egui::epaint::ClippedShape;
use egui::{Event, Id, PointerButton, Pos2, RawInput, Rect, TouchPhase, Ui, Vec2};

/// One frame's time step.
const FRAME: f64 = 1.0 / 60.0;

pub struct Harness {
    pub ctx: egui::Context,
    screen: Rect,
    time: f64,
}

impl Harness {
    pub fn new(screen: Rect) -> Self {
        Self {
            ctx: egui::Context::default(),
            screen,
            time: 0.0,
        }
    }

    /// Runs one frame of `run` with `events`; the shapes it painted.
    pub fn frame(
        &mut self,
        events: Vec<Event>,
        run: &mut impl FnMut(&mut Ui),
    ) -> Vec<ClippedShape> {
        self.time += FRAME;
        let input = RawInput {
            screen_rect: Some(self.screen),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| run(ui));
        output.textures_delta.clear();
        output.shapes
    }

    /// Runs `n` frames without input; the last frame's shapes.
    pub fn settle(&mut self, n: usize, run: &mut impl FnMut(&mut Ui)) -> Vec<ClippedShape> {
        let mut shapes = Vec::new();
        for _ in 0..n {
            shapes = self.frame(Vec::new(), run);
        }
        shapes
    }

    /// Moves the pointer to `pos`, pressing or releasing the primary button
    /// there when `pressed` is set; the frame's shapes.
    pub fn pointer(
        &mut self,
        pos: Pos2,
        pressed: Option<bool>,
        run: &mut impl FnMut(&mut Ui),
    ) -> Vec<ClippedShape> {
        let mut events = vec![Event::PointerMoved(pos)];
        events.extend(pressed.map(|pressed| primary(pos, pressed)));
        self.frame(events, run)
    }

    /// Hovers `pos`, then presses and releases there.
    pub fn click(&mut self, pos: Pos2, run: &mut impl FnMut(&mut Ui)) {
        self.pointer(pos, None, run);
        self.pointer(pos, Some(true), run);
        self.pointer(pos, Some(false), run);
    }

    /// A finger's tap at `pos`, the way a touch screen reports it: no hover
    /// before the touch, and the pointer gone once the finger lifts.
    pub fn tap(&mut self, pos: Pos2, run: &mut impl FnMut(&mut Ui)) {
        let touch = |phase| Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId(0),
            phase,
            pos,
            force: None,
        };
        let down = vec![
            touch(egui::TouchPhase::Start),
            Event::PointerMoved(pos),
            primary(pos, true),
        ];
        self.frame(down, run);
        let up = vec![
            touch(egui::TouchPhase::End),
            primary(pos, false),
            Event::PointerGone,
        ];
        self.frame(up, run);
        self.frame(Vec::new(), run);
    }

    /// One frame of `(finger, phase, position)` touches, the way eframe
    /// reports them: finger 0 also drives the pointer, pressed when it lands
    /// and released (and gone) when it lifts.
    pub fn touch(&mut self, touches: &[(u64, TouchPhase, Pos2)], run: &mut impl FnMut(&mut Ui)) {
        let mut events = Vec::new();
        for &(finger, phase, pos) in touches {
            events.push(Event::Touch {
                device_id: egui::TouchDeviceId(0),
                id: egui::TouchId(finger),
                phase,
                pos,
                force: None,
            });
            if finger != 0 {
                continue;
            }
            match phase {
                TouchPhase::Start => events.extend([Event::PointerMoved(pos), primary(pos, true)]),
                TouchPhase::Move => events.push(Event::PointerMoved(pos)),
                TouchPhase::End | TouchPhase::Cancel => {
                    events.extend([primary(pos, false), Event::PointerGone]);
                }
            }
        }
        self.frame(events, run);
    }

    /// Two fingers side by side about `center`, `from` apart, spreading to
    /// `to` apart while their center moves by `shift`, over ten frames.
    pub fn pinch(
        &mut self,
        center: Pos2,
        (from, to): (f32, f32),
        shift: Vec2,
        run: &mut impl FnMut(&mut Ui),
    ) {
        let both = |t: f32, phase| {
            let half = Vec2::X * (from + (to - from) * t) / 2.0;
            let mid = center + shift * t;
            [(0, phase, mid - half), (1, phase, mid + half)]
        };
        self.touch(&both(0.0, TouchPhase::Start), run);
        for step in 1..=10 {
            self.touch(&both(step as f32 / 10.0, TouchPhase::Move), run);
        }
        self.touch(&both(1.0, TouchPhase::End), run);
        self.frame(Vec::new(), run);
    }

    /// Presses at `from`, moves by `delta` over ten frames and releases.
    pub fn drag(&mut self, from: Pos2, delta: Vec2, run: &mut impl FnMut(&mut Ui)) {
        self.pointer(from, None, run);
        self.pointer(from, Some(true), run);
        for step in 1..=10 {
            self.pointer(from + delta * (step as f32 / 10.0), None, run);
        }
        self.pointer(from + delta, Some(false), run);
        self.frame(Vec::new(), run);
    }

    /// The rect of the area `id`, shown last frame.
    pub fn area(&self, id: Id) -> Rect {
        self.ctx
            .memory(|m| m.area_rect(id))
            .unwrap_or_else(|| panic!("area {id:?} is shown"))
    }

    /// The id of the topmost area under `pos`.
    pub fn top_at(&self, pos: Pos2) -> Option<Id> {
        self.ctx.layer_id_at(pos).map(|layer| layer.id)
    }
}

/// The primary button pressed or released at `pos`.
fn primary(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    }
}
