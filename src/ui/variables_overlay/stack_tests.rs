//! Multiscale variables in the tree: one row per stack, its chevron folding
//! down a row per level; the row selects Auto, a level row pins it.

use egui::{Event, Key, PointerButton, Pos2, RawInput, pos2};

use super::nav::{NodeKey, row_id, visible_row_ids};
use super::stack::stack_state;
use super::tree::{VariableTreeContext, render_tree_group};
use crate::app::layers::LevelChoice;
use crate::data::multiscale::{Multiscale, ScaleLevel, link};
use crate::data::{VariableInfo, VariableTreeGroup};
use crate::ui::key_focus;

/// `lat` and the three levels of `t2m` (64, 32 and 16 cells a side).
fn fixture() -> (Vec<VariableInfo>, VariableTreeGroup) {
    let var = |name: &str, n: u64| VariableInfo {
        name: name.into(),
        shape: vec![n, n],
        dimension_names: vec!["y".into(), "x".into()],
        ..Default::default()
    };
    let mut variables = vec![
        var("lat", 4),
        var("0/t2m", 64),
        var("1/t2m", 32),
        var("2/t2m", 16),
    ];
    let levels = (0..3)
        .map(|l| {
            let f = f64::from(1u32 << l);
            ScaleLevel {
                path: format!("{l}/t2m"),
                shape: vec![64 >> l, 64 >> l],
                factor: vec![f, f],
                offset: vec![0.0, 0.0],
            }
        })
        .collect();
    let dims = vec!["y".into(), "x".into()];
    let set = Multiscale::new("t2m".into(), dims, levels);
    link(set.expect("stack"), &mut variables);
    let root = VariableTreeGroup::build_tree_from_variables(&variables);
    (variables, root)
}

struct Frame {
    origin: Pos2,
    row_step: f32,
    selected: Option<(usize, LevelChoice)>,
}

fn frame(ctx: &egui::Context, events: Vec<Event>) -> Frame {
    let (variables, root) = fixture();
    // A tenth of a second a frame: past the fold-down animation, short of a long press.
    let input = RawInput {
        events,
        time: Some(ctx.input(|i| i.time) + 0.1),
        ..Default::default()
    };
    let mut out = None;
    let mut output = ctx.run_ui(input, |ui| {
        let origin = ui.cursor().min;
        let row_step = ui.spacing().interact_size.y.max(20.0) + ui.spacing().item_spacing.y;
        let mut tree = VariableTreeContext {
            variables: &variables,
            selected_idx: usize::MAX,
            selected_level: LevelChoice::Auto,
            search_active: false,
            newly_selected_idx: None,
            newly_selected_level: LevelChoice::Auto,
            search_id: egui::Id::new("search"),
            search_jump: None,
        };
        render_tree_group(ui, &root, &mut tree);
        let level = tree.newly_selected_level;
        out = Some(Frame {
            origin,
            row_step,
            selected: tree.newly_selected_idx.map(|idx| (idx, level)),
        });
    });
    output.textures_delta.clear();
    out.expect("frame ran")
}

fn click(ctx: &egui::Context, pos: Pos2) -> Frame {
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(ctx, vec![Event::PointerMoved(pos)]);
    frame(ctx, vec![button(true)]);
    frame(ctx, vec![button(false)])
}

fn press(ctx: &egui::Context, key: Key) {
    let event = Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    };
    frame(ctx, vec![event]);
}

fn is_open(ctx: &egui::Context) -> bool {
    stack_state(ctx, 1, false).is_open()
}

/// The point `x` px into row `row`.
fn at(f: &Frame, row: f32, x: f32) -> Pos2 {
    pos2(f.origin.x + x, f.origin.y + f.row_step * (row + 0.5))
}

#[test]
fn a_stack_is_listed_once_under_its_name() {
    let (variables, root) = fixture();
    assert_eq!(root.variable_indices, [0, 1]);
    assert!(root.subgroups.is_empty(), "no level folders");
    assert_eq!(variables[1].display_name(), "t2m");
}

#[test]
fn the_row_selects_auto_and_its_chevron_folds_the_levels() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new());
    let f = click(&ctx, at(&f, 1.0, 80.0));
    assert_eq!(f.selected, Some((1, LevelChoice::Auto)));
    assert!(!is_open(&ctx), "selecting does not unfold");

    let f = click(&ctx, at(&f, 1.0, 6.0));
    assert_eq!(f.selected, None, "the chevron only folds");
    assert!(is_open(&ctx));
    frame(&ctx, Vec::new());

    let f = click(&ctx, at(&f, 3.0, 80.0));
    assert_eq!(f.selected, Some((1, LevelChoice::Pinned(1))));
}

#[test]
fn arrow_keys_unfold_and_walk_the_levels() {
    let ctx = egui::Context::default();
    frame(&ctx, Vec::new());
    key_focus::request(&ctx, row_id(NodeKey::Variable(1), false));
    frame(&ctx, Vec::new());
    press(&ctx, Key::ArrowRight);
    assert!(is_open(&ctx));
    press(&ctx, Key::ArrowDown);
    let level0 = row_id(NodeKey::Level(1, 0), false);
    assert_eq!(ctx.memory(|m| m.focused()), Some(level0));
    press(&ctx, Key::ArrowLeft);
    let stack = row_id(NodeKey::Variable(1), false);
    assert_eq!(
        ctx.memory(|m| m.focused()),
        Some(stack),
        "left goes to the stack"
    );
    press(&ctx, Key::ArrowLeft);
    assert!(!is_open(&ctx));
}

#[test]
fn navigation_lists_the_drawn_level_rows() {
    let ctx = egui::Context::default();
    let mut state = stack_state(&ctx, 1, false);
    state.set_open(true);
    state.store(&ctx);
    let f = frame(&ctx, Vec::new());
    let (variables, root) = fixture();
    let ids = visible_row_ids(&ctx, &root, &variables, false);
    assert_eq!(ids.len(), 5, "lat, t2m and its three levels");
    for (i, id) in ids.iter().enumerate() {
        let rect = ctx.read_response(*id).expect("drawn").rect;
        let top = f.origin.y + f.row_step * i as f32;
        assert!((rect.top() - top).abs() < 0.5, "row {i} at {}", rect.top());
    }
}
