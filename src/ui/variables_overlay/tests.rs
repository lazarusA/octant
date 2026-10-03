//! Headless egui harness for the variable tree, plus mouse interaction tests.
//! Keyboard navigation tests live in `nav_tests.rs`.

use super::nav::{NodeKey, folder_id, row_id};
use super::row::RowKind;
use super::search_jump;
use super::tree::{VariableTreeContext, render_tree_group};
use crate::data::{VariableInfo, VariableTreeGroup};
use crate::ui::icons::{Icon, UiIconExt};
use crate::ui::key_focus;
use egui::collapsing_header::CollapsingState;
use egui::{Event, Id, Key, Modifiers, PointerButton, Pos2, RawInput, pos2};

fn var(name: &str) -> VariableInfo {
    VariableInfo {
        name: name.into(),
        ..Default::default()
    }
}

fn group(path: &str, vars: Vec<usize>, subgroups: Vec<VariableTreeGroup>) -> VariableTreeGroup {
    VariableTreeGroup {
        name: path.rsplit('/').next().unwrap_or(path).into(),
        full_path: path.into(),
        variable_indices: vars,
        subgroups,
    }
}

/// Display order with every folder open:
/// `/` > lat(0), ocean > sst(1), ocean/deep > temp(2), land > lai(3).
fn fixture() -> (Vec<VariableInfo>, VariableTreeGroup) {
    let variables = ["lat", "ocean/sst", "ocean/deep/temp", "land/lai"].map(var);
    let deep = group("ocean/deep", vec![2], Vec::new());
    let ocean = group("ocean", vec![1], vec![deep]);
    let land = group("land", vec![3], Vec::new());
    let root = group("", vec![0], vec![ocean, land]);
    (variables.to_vec(), root)
}

pub(super) struct Frame {
    pub origin: Pos2,
    pub row_step: f32,
    pub ocean_id: Id,
    pub selected: Option<usize>,
    pub search_id: Id,
}

/// Run one frame of the search field and tree, feeding `events` as input.
pub(super) fn frame(ctx: &egui::Context, events: Vec<Event>, search_active: bool) -> Frame {
    let (variables, root) = fixture();
    let input = RawInput {
        events,
        ..Default::default()
    };
    let mut out = None;
    let mut output = ctx.run_ui(input, |ui| {
        let mut text = String::new();
        let search = ui.search_field_response(&mut text, "Search", None);
        let jump = search_jump(ui, &search);
        let origin = ui.cursor().min;
        let row_step = ui.spacing().interact_size.y.max(20.0) + ui.spacing().item_spacing.y;
        let mut tree_ctx = VariableTreeContext {
            variables: &variables,
            selected_idx: usize::MAX,
            search_active,
            newly_selected_idx: None,
            search_id: Some(search.id),
            search_jump: jump,
        };
        render_tree_group(ui, &root, &mut tree_ctx, true);
        out = Some(Frame {
            origin,
            row_step,
            ocean_id: folder_id("ocean", search_active),
            selected: tree_ctx.newly_selected_idx,
            search_id: search.id,
        });
    });
    output.textures_delta.clear();
    out.expect("frame ran")
}

/// Click at `pos` over three frames and return the frame that saw the release.
pub(super) fn click(ctx: &egui::Context, pos: Pos2, search_active: bool) -> Frame {
    let button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(ctx, vec![Event::PointerMoved(pos)], search_active);
    frame(ctx, vec![button(true)], search_active);
    frame(ctx, vec![button(false)], search_active)
}

/// Press `key` for one frame.
pub(super) fn press(ctx: &egui::Context, key: Key, search_active: bool) -> Frame {
    let event = Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    frame(ctx, vec![event], search_active)
}

/// Focus the row of `key`, as a click or earlier navigation would.
pub(super) fn focus(ctx: &egui::Context, key: NodeKey<'_>, search_active: bool) {
    key_focus::request(ctx, row_id(key, search_active));
}

pub(super) fn focused(ctx: &egui::Context) -> Option<Id> {
    ctx.memory(|m| m.focused())
}

pub(super) fn is_open(ctx: &egui::Context, id: Id) -> bool {
    CollapsingState::load(ctx, id).is_some_and(|s| s.is_open())
}

#[test]
fn folder_row_icons_follow_open_state() {
    assert_eq!(RowKind::Folder { open: false }.icon(), Icon::Folder);
    assert_eq!(RowKind::Folder { open: true }.icon(), Icon::FolderOpen);
    assert_eq!(
        RowKind::Folder { open: true }.chevron(),
        Some(Icon::ChevronDown)
    );
    assert_eq!(RowKind::Variable.chevron(), None);
}

#[test]
fn clicking_folder_name_toggles_it() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), false);
    assert!(!is_open(&ctx, f.ocean_id), "folders start closed");

    // Row 0 is the "/" folder, row 1 is `ocean`; click its name, far from the chevron.
    let name = pos2(f.origin.x + 80.0, f.origin.y + f.row_step * 1.5);
    let f = click(&ctx, name, false);
    assert!(
        is_open(&ctx, f.ocean_id),
        "click on the name opens the folder"
    );

    click(&ctx, name, false);
    assert!(!is_open(&ctx, f.ocean_id), "second click closes it");
}

#[test]
fn clicking_variable_icon_selects_it() {
    let ctx = egui::Context::default();
    let f = frame(&ctx, Vec::new(), false);
    click(
        &ctx,
        pos2(f.origin.x + 80.0, f.origin.y + f.row_step * 1.5),
        false,
    );

    // Row 2 is `ocean/sst`, indented under its folder; click over its icon.
    let icon = pos2(f.origin.x + 18.0 + 24.0, f.origin.y + f.row_step * 2.5);
    let f = click(&ctx, icon, false);
    assert_eq!(f.selected, Some(1));
}

#[test]
fn search_expansion_does_not_leak_into_browsing() {
    let ctx = egui::Context::default();
    let browse = frame(&ctx, Vec::new(), false);
    let search = frame(&ctx, Vec::new(), true);
    assert!(
        is_open(&ctx, search.ocean_id),
        "search results start expanded"
    );

    let browse_again = frame(&ctx, Vec::new(), false);
    assert_eq!(browse.ocean_id, browse_again.ocean_id);
    assert!(
        !is_open(&ctx, browse_again.ocean_id),
        "clearing search restores browse state"
    );
}
