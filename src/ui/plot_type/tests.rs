//! Plot types offered for a variable, and the centered picker by keyboard:
//! a number key picks, Left, Right, Home, End and Enter pick, Escape leaves
//! it alone.

use super::options::{ORDER, plot_options};
use super::picker::show_plot_type_picker;
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::keyboard::dispatch;
use crate::ui::panel_width_tests::plotted;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Ui, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

#[test]
fn without_a_3d_variable_volumes_are_offered() {
    let app = OctantApp::default();
    let options = plot_options(&app);
    assert_eq!(options.map(|o| o.plot_type), ORDER);
    let blocked = |t: PlotType| {
        options
            .iter()
            .find(|o| o.plot_type == t)
            .and_then(|o| o.blocked)
    };
    assert_eq!(blocked(PlotType::Heatmap), None);
    assert_eq!(blocked(PlotType::Line), None);
    // A 2D variable draws as a one-slice volume or point cloud.
    assert_eq!(blocked(PlotType::Volume), None);
    assert_eq!(blocked(PlotType::PointCloud), None);
}

fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| {
        dispatch(app, ui.ctx(), true);
        show_plot_type_picker(app, ui.ctx());
    }
}

fn press(h: &mut Harness, app: &mut OctantApp, key: Key) {
    h.press(key, Modifiers::NONE, &mut run(app));
}

#[test]
fn p_opens_the_picker_and_a_number_key_picks() {
    let mut app = plotted(PlotType::Heatmap);
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::P);
    assert!(app.layout.show_plot_picker);
    press(&mut h, &mut app, Key::Num2);
    assert_eq!(app.selected.plot_type, PlotType::Line);
    assert!(!app.layout.show_plot_picker, "picking closes it");
}

#[test]
fn arrows_and_enter_pick_and_escape_changes_nothing() {
    let mut app = plotted(PlotType::Heatmap);
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::P);
    press(&mut h, &mut app, Key::Escape);
    assert!(!app.layout.show_plot_picker);
    assert_eq!(app.selected.plot_type, PlotType::Heatmap);

    // Focus starts on the current type; End and Home go to the ends, Down
    // does nothing in the row, Right moves to the next tile.
    press(&mut h, &mut app, Key::P);
    for key in [Key::End, Key::Home, Key::ArrowDown, Key::ArrowRight] {
        press(&mut h, &mut app, key);
    }
    press(&mut h, &mut app, Key::Enter);
    assert_eq!(app.selected.plot_type, PlotType::Line);
    assert!(!app.layout.show_plot_picker);
}

#[test]
fn a_click_outside_closes_the_picker() {
    let mut app = plotted(PlotType::Heatmap);
    let mut h = Harness::new(SCREEN);
    h.settle(2, &mut run(&mut app));
    press(&mut h, &mut app, Key::P);
    h.click(pos2(5.0, 5.0), &mut run(&mut app));
    assert!(!app.layout.show_plot_picker);
    assert_eq!(app.selected.plot_type, PlotType::Heatmap);
}

#[test]
fn the_picker_keeps_its_width_in_every_language() {
    let make_app = || {
        let mut app = plotted(PlotType::Heatmap);
        app.layout.show_plot_picker = true;
        app
    };
    let width = super::picker::WIDTH;
    crate::ui::panel_width_tests::assert_window_width(
        "plot_type_picker",
        width,
        make_app,
        show_plot_type_picker,
    );
}

#[test]
fn a_volume_whose_data_is_gone_falls_back_to_a_heatmap_without_a_z_role() {
    use crate::app::SpatialRole;
    use crate::app::test_support::{memory_app, selection_of};
    let (mut app, meta) = memory_app();
    app.selected = selection_of(&mut app, &meta, "t2m");
    app.switch_plot_type(PlotType::Volume);
    let has_z =
        |app: &OctantApp| (app.selected.dim_config.iter()).any(|c| c.spatial == SpatialRole::Z);
    assert!(has_z(&app), "the volume stacks a dimension along Z");
    // Plotted, but its volume is gone (released) and nothing loads.
    app.sync_plotted_state_from_selected();
    app.layers.base.data.volume = None;

    super::options::fall_back_to_heatmap(&mut app);

    assert_eq!(app.selected.plot_type, PlotType::Heatmap);
    assert!(!has_z(&app), "a heatmap has no Z");
}
