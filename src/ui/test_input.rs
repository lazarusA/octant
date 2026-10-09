//! Headless egui frames for UI tests: a context with its own clock (so
//! double-clicks and fades follow the frames), pointer moves, clicks and
//! drags, and the areas' rects. Each call takes the frame's UI as a closure;
//! build it per call (`&mut run(&mut app)`) so the app stays free between
//! frames.

use egui::epaint::ClippedShape;
use egui::{Event, Id, PointerButton, Pos2, RawInput, Rect, Ui, Vec2};

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
        if let Some(pressed) = pressed {
            events.push(Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            });
        }
        self.frame(events, run)
    }

    /// Hovers `pos`, then presses and releases there.
    pub fn click(&mut self, pos: Pos2, run: &mut impl FnMut(&mut Ui)) {
        self.pointer(pos, None, run);
        self.pointer(pos, Some(true), run);
        self.pointer(pos, Some(false), run);
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
